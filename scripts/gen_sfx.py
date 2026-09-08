#!/usr/bin/env python3
"""Procedural generator for dx-rpg's combat sound effects.

Renders the sfx under `assets/audio/sfx/` that we author ourselves (as opposed to
the CC0 packs listed in `assets/audio/License.txt`) from the synthesis recipes
below, so a sound can be re-tuned by editing a number here and re-running:

    python3 scripts/gen_sfx.py            # writes assets/audio/sfx/*.ogg
    python3 scripts/gen_sfx.py --out /tmp # audition somewhere else first

Needs only the Python standard library plus `ffmpeg` on PATH (for WAV -> Ogg
Vorbis, the format the audio bridge in `src/audio.rs` loads). Rendering is
deterministic — two runs decode to bit-identical PCM — but the .ogg containers
differ byte for byte, because libvorbis stamps a random stream serial into each
one. Re-running therefore always shows up as a binary diff; that is the
container, not the sound.

An attack sounds like *what it costs to cast* (see `src/sfx_cue.rs`), so a given
attack always sounds the same and its family is audible before you read the log:

* `strike`  — costs nothing: the basic attack ("Charge"). Heard more than
              anything else in the game, so it is kept short and sharp: a snap, a
              slap, and a body that is gone before the next one.
* `arcane`  — costs mana: a bright inharmonic burst over a low thump, so a spell
              still lands as a hit and not just a chime.
* `heavy`   — costs vigour: a slow, massive crunch — the darkest and longest of
              the impacts by a wide margin, where `strike` is the quickest and
              brightest. The physical-effort counterpart to `arcane`.
* `rage`    — costs berserk: a gritty tear into a heavy slam, with a second
              smaller hit right behind it. Grittiest of the four on purpose.
* `critical`— not a family of its own: bright shards, with no low end and no
              transient, layered *over* whichever family sound just played, so a
              crit reads as the same attack hitting harder.

And the non-damage cues:

* `heal`    — an instant cure: a warm rising fifth on soft bells. Shorter and
              rounder than `buff`, which has to stay distinguishable from it.
* `potion`  — must read as *relief*, not as glassware: a cork, three gulps, then
              a warm rising major arpeggio over a low root. The rising major
              third/fifth is what makes it feel like survival rather than like
              an inventory click.
* `buff`    — a lasting blessing landing on an ally (regen, shields, war cries).
              An airy upward whoosh into two crystalline chimes; deliberately
              brighter and longer than `heal` so a sustained buff is
              distinguishable from an instant cure.
* `debuff`  — the mirror image: a downward whoosh into a dark tritone, for a
              curse that lands without dealing damage.
"""

from __future__ import annotations

import argparse
import math
import os
import random
import shutil
import struct
import subprocess
import sys
import tempfile
import wave

SR = 44100

Signal = list  # list[float], mono, one sample per frame


# --------------------------------------------------------------------------
# building blocks
# --------------------------------------------------------------------------


def buf(duration: float) -> Signal:
    return [0.0] * int(duration * SR)


def mix_into(dst: Signal, src: Signal, at: float = 0.0, gain: float = 1.0) -> Signal:
    """Adds `src` into `dst` starting at `at` seconds, extending `dst` if needed."""
    start = int(at * SR)
    if len(dst) < start + len(src):
        dst.extend([0.0] * (start + len(src) - len(dst)))
    for i, s in enumerate(src):
        dst[start + i] += s * gain
    return dst


def tone(
    duration: float,
    freq: float,
    *,
    amp: float = 1.0,
    attack: float = 0.004,
    decay: float = 0.2,
    partials: tuple[tuple[float, float], ...] = ((1.0, 1.0),),
    bend: tuple[float, float] | None = None,
    vibrato: tuple[float, float] = (0.0, 0.0),
) -> Signal:
    """An enveloped additive tone.

    `partials` are (frequency ratio, amplitude) pairs; `bend` is a
    (start multiplier, time constant) pitch glide decaying back to `freq`;
    `vibrato` is (rate Hz, depth as a fraction of the frequency); `decay` is the
    exponential time constant of the amplitude tail, not its total length.
    """
    n = int(duration * SR)
    out = [0.0] * n
    phases = [0.0] * len(partials)
    for i in range(n):
        t = i / SR
        env = (1.0 - math.exp(-t / max(attack, 1e-6))) * math.exp(-t / decay)
        f = freq
        if bend:
            f *= 1.0 + (bend[0] - 1.0) * math.exp(-t / bend[1])
        if vibrato[1]:
            f *= 1.0 + vibrato[1] * math.sin(2 * math.pi * vibrato[0] * t)
        acc = 0.0
        for p, (ratio, p_amp) in enumerate(partials):
            phases[p] += 2 * math.pi * f * ratio / SR
            acc += p_amp * math.sin(phases[p])
        out[i] = acc * env * amp
    return out


def noise(duration: float, *, amp: float = 1.0, seed: int = 0) -> Signal:
    rng = random.Random(seed)
    return [rng.uniform(-1.0, 1.0) * amp for _ in range(int(duration * SR))]


def svf(x: Signal, freq, q: float = 0.7, mode: str = "band") -> Signal:
    """Chamberlin state-variable filter; `freq` may be a constant or a function
    of time in seconds, which is what makes the swept whooshes possible."""
    low = band = 0.0
    out = [0.0] * len(x)
    freq_at = freq if callable(freq) else (lambda _t: freq)
    for i, s in enumerate(x):
        # Clamped well below SR/4, where this topology stops being stable.
        f = 2.0 * math.sin(math.pi * min(freq_at(i / SR), SR * 0.22) / SR)
        high = s - low - q * band
        band += f * high
        low += f * band
        out[i] = {"low": low, "band": band, "high": high}[mode]
    return out


def envelope(x: Signal, *, attack: float = 0.002, decay: float = 0.1, hold: float = 0.0) -> Signal:
    out = [0.0] * len(x)
    for i, s in enumerate(x):
        t = i / SR
        rise = 1.0 - math.exp(-t / max(attack, 1e-6))
        fall = 1.0 if t < hold else math.exp(-(t - hold) / decay)
        out[i] = s * rise * fall
    return out


def swell(x: Signal, *, attack: float, release: float) -> Signal:
    """A slow fade-in / fade-out shape, for pads and air layers."""
    n = len(x)
    out = [0.0] * n
    a = max(int(attack * SR), 1)
    r = max(int(release * SR), 1)
    for i, s in enumerate(x):
        rise = min(i / a, 1.0)
        fall = min((n - i) / r, 1.0)
        out[i] = s * rise * fall * fall
    return out


def reverb(x: Signal, *, wet: float = 0.3, decay: float = 0.5, tail: float = 0.8, seed: int = 0) -> Signal:
    """Schroeder reverb (four combs into two allpasses). Cheap, and all these
    sounds need is a room to stop them from sounding like a dry beep."""
    src = list(x) + [0.0] * int(tail * SR)
    rng = random.Random(seed)
    acc = [0.0] * len(src)
    for delay_ms in (29.7, 37.1, 41.1, 43.7):
        d = int(delay_ms * 0.001 * SR * rng.uniform(0.97, 1.03))
        fb = math.exp(-3.0 * (d / SR) / decay)
        line = [0.0] * len(src)
        for i, s in enumerate(src):
            line[i] = s + (line[i - d] * fb if i >= d else 0.0)
        for i, s in enumerate(line):
            acc[i] += s * 0.25
    for delay_ms, g in ((5.0, 0.7), (1.7, 0.7)):
        d = int(delay_ms * 0.001 * SR)
        line = [0.0] * len(acc)
        for i, s in enumerate(acc):
            delayed = line[i - d] if i >= d else 0.0
            line[i] = -g * s + delayed + g * (acc[i - d] if i >= d else 0.0)
        acc = line
    return [(1.0 - wet) * (src[i] if i < len(src) else 0.0) + wet * acc[i] for i in range(len(acc))]


def saturate(x: Signal, drive: float = 1.0) -> Signal:
    """Soft clipping — adds the harmonics that make an impact feel like it hit
    something, and keeps peaks in check for free."""
    return [math.tanh(s * drive) / math.tanh(drive) for s in x]


def normalize(x: Signal, peak_db: float = -1.5) -> Signal:
    peak = max((abs(s) for s in x), default=0.0)
    if peak == 0.0:
        return x
    return [s * (10 ** (peak_db / 20.0)) / peak for s in x]


def fade_out(x: Signal, seconds: float = 0.030) -> Signal:
    n = min(int(seconds * SR), len(x))
    for i in range(n):
        x[len(x) - n + i] *= 1.0 - i / n
    return x


def trim_silence(x: Signal, threshold: float = 1.5e-3) -> Signal:
    """Drops an inaudible tail so the file (and the sound's felt length) stops
    when the sound does."""
    end = len(x)
    while end > 1 and abs(x[end - 1]) < threshold:
        end -= 1
    return x[:end]


def widen(x: Signal, spread: float = 0.0) -> list[tuple[float, float]]:
    """Mono to stereo. `spread` delays the right channel by a few samples, which
    gives the magical sounds a little width without hurting a mono mixdown."""
    d = int(spread * SR)
    return [(x[i], x[i - d] if i >= d else 0.0) for i in range(len(x))]


def write_wav(path: str, stereo: list[tuple[float, float]]) -> None:
    with wave.open(path, "wb") as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(SR)
        frames = bytearray()
        for left, right in stereo:
            frames += struct.pack(
                "<hh",
                int(max(-1.0, min(1.0, left)) * 32767),
                int(max(-1.0, min(1.0, right)) * 32767),
            )
        w.writeframes(bytes(frames))


# --------------------------------------------------------------------------
# the sounds
# --------------------------------------------------------------------------


def make_strike() -> Signal:
    """No-cost basic strike: a snap, a slap, and a body that is gone in a quarter
    of a second.

    Heard more than anything else in the game, so it starts on the impact itself.
    An earlier version led with a swing, but once the file was normalised to the
    impact's peak that swing sat 34 dB down — inaudible, and delaying the hit the
    player asked for by 55ms."""
    out = buf(0.30)
    # The snap: very short, very bright. This is what makes the hit read as sharp
    # rather than soft, and it has to come first.
    mix_into(out, envelope(svf(noise(0.03, seed=52), 3400.0, q=1.6), attack=0.0002, decay=0.0035), 0.0, 0.55)
    # The slap: mid weight, right behind the snap.
    mix_into(out, envelope(svf(noise(0.07, seed=53), 1500.0, q=0.9), attack=0.0004, decay=0.014), 0.0, 0.50)
    # Body: the fast downward bend is what makes it a blow rather than a beep.
    mix_into(out, tone(0.24, 96.0, attack=0.0008, decay=0.040, bend=(4.2, 0.014), partials=((1.0, 1.0), (2.0, 0.22), (3.0, 0.08))), 0.0, 0.95)
    mix_into(out, tone(0.24, 52.0, attack=0.003, decay=0.065), 0.0, 0.50)
    return saturate(out, 1.5)


def make_arcane() -> Signal:
    """Mana-cost attack: a bright inharmonic burst over a low thump.

    Starts on the burst. The rising charge that used to lead it measured 34 dB
    below the burst once the file was normalised — inaudible, and it pushed the
    spell 120ms behind the button that cast it."""
    out = buf(0.85)
    mix_into(out, envelope(svf(noise(0.10, seed=62), 3200.0, q=0.9), attack=0.0005, decay=0.026), 0.0, 0.55)
    burst = buf(0.70)
    for freq, amp in ((523.25, 1.0), (783.99, 0.55), (1174.66, 0.30)):
        mix_into(burst, tone(0.65, freq, attack=0.003, decay=0.17, partials=((1.0, 1.0), (2.76, 0.18)), bend=(1.25, 0.05)), 0.0, 0.34 * amp)
    # The low thump underneath: without it a spell reads as a chime, not a hit.
    mix_into(burst, tone(0.30, 82.0, attack=0.002, decay=0.065, bend=(2.2, 0.020), partials=((1.0, 1.0), (2.0, 0.22))), 0.0, 0.95)
    mix_into(out, reverb(burst, wet=0.22, decay=0.40, tail=0.30, seed=6), 0.0)
    return saturate(out, 2.2)


def make_heavy() -> Signal:
    """Vigour-cost attack: a slow, massive crunch.

    Weight comes from three things and this leans on all of them: low-frequency
    energy, a long decay, and an attack that is not instant — heavy things take
    time to start moving and much longer to stop. It is the darkest and by far the
    longest of the four impacts, where `strike` is the quickest and brightest.

    The body sits at 52Hz, but its partials at 104, 156 and 208Hz, plus the clank
    above them, are what carry the weight on a laptop or phone speaker — neither
    reproduces the 26Hz sub at all. The heavy drive at the end is deliberate: it
    compresses the transient, which lets the sustained low end come up under
    normalisation. Measured across a sweep it buys about 3dB in both bands that
    convey weight, for 0.5dB in the 250Hz-2k band that works against it.

    Going lower than this stops helping — at a 130-140Hz crunch the fundamental
    falls out of the range small speakers reproduce and the sound gets *less*
    weighty, not more."""
    out = buf(1.20)
    # Crunch: centred low and kept narrow, so little of it spills into the
    # 250Hz-2k band that makes an impact read as light and clicky.
    mix_into(out, envelope(svf(noise(0.30, seed=72), 160.0, q=1.3), attack=0.0010, decay=0.100), 0.0, 0.80)
    # Barely any top: just enough to mark the moment of contact. A hard click here
    # is the single fastest way to make a heavy sound read as light.
    mix_into(out, envelope(svf(noise(0.05, seed=73), 2200.0, q=1.0, mode="high"), attack=0.0005, decay=0.006), 0.0, 0.07)
    # Body: low, slow to start, slow to bend, slow to die.
    mix_into(out, tone(1.15, 52.0, attack=0.004, decay=0.260, bend=(2.8, 0.075), partials=((1.0, 1.0), (2.0, 0.35), (3.0, 0.12), (4.0, 0.05))), 0.0, 1.00)
    # An exact octave below the body, so its second harmonic reinforces 52Hz
    # instead of beating against it.
    mix_into(out, tone(1.15, 26.0, attack=0.008, decay=0.320), 0.0, 0.90)
    # A low clank rather than a ring: the steel, without the brightness, and short
    # enough to stay well inside the body's decay.
    for freq, amp, decay in ((165.0, 1.0, 0.110), (247.0, 0.45, 0.085), (392.0, 0.10, 0.050)):
        mix_into(out, tone(0.40, freq, attack=0.002, decay=decay, partials=((1.0, 1.0), (2.41, 0.08))), 0.0, 0.15 * amp)
    return saturate(out, 3.2)


def make_rage() -> Signal:
    """Berserk-cost attack: a gritty tear straight into a heavy slam, with a second
    smaller hit right behind it.

    Nothing here sustains. The version this replaces opened on two detuned low
    tones beating against each other, which read as a drone rather than as fury;
    the double hit carries the savagery instead."""
    out = buf(0.55)
    # Rush in: noise falling fast, short enough to only lead the hit.
    rush = svf(noise(0.055, seed=81), lambda t: 2800.0 - 2100.0 * (t / 0.055), q=0.6)
    mix_into(out, envelope(rush, attack=0.012, decay=0.020), 0.0, 0.25)

    at = 0.055
    # The tear: saturated band noise sliding down. The character of the family.
    tear = svf(noise(0.13, seed=82), lambda t: 1100.0 - 700.0 * min(t / 0.09, 1.0), q=0.5)
    mix_into(out, saturate(envelope(tear, attack=0.001, decay=0.055), 2.6), at, 0.50)
    mix_into(out, tone(0.40, 62.0, attack=0.001, decay=0.085, bend=(4.0, 0.022), partials=((1.0, 1.0), (2.0, 0.35), (3.0, 0.18))), at, 1.00)
    mix_into(out, tone(0.40, 38.0, attack=0.004, decay=0.130), at, 0.62)

    # The follow-through, close enough behind to read as one savage action.
    at2 = at + 0.085
    mix_into(out, envelope(svf(noise(0.07, seed=83), 1600.0, q=0.9), attack=0.0004, decay=0.014), at2, 0.30)
    mix_into(out, tone(0.24, 74.0, attack=0.001, decay=0.050, bend=(3.0, 0.016), partials=((1.0, 1.0), (2.0, 0.30))), at2, 0.45)
    return saturate(out, 1.5)


def make_critical() -> Signal:
    """Crit accent: three bright shards ringing out over the family impact
    `classify_attack` pairs it with. No low end of its own, and no noise burst in
    front of the shards either — that burst fought the transient of whichever hit
    it was layered on."""
    out = buf(0.55)
    shards = buf(0.45)
    for at, freq in ((0.0, 1567.98), (0.035, 2093.00), (0.070, 2637.02)):
        mix_into(shards, tone(0.40, freq, attack=0.001, decay=0.10, partials=((1.0, 1.0), (2.76, 0.25))), at, 0.30)
    mix_into(out, reverb(shards, wet=0.30, decay=0.35, tail=0.25, seed=10), 0.0)
    return saturate(out, 1.8)


def make_heal() -> Signal:
    """An instant cure: a warm rising fifth on soft bells, over a low root."""
    out = buf(0.95)
    mix_into(out, swell(svf(noise(0.50, seed=91), 4000.0, q=0.5, mode="high"), attack=0.10, release=0.30), 0.0, 0.06)
    bells = buf(0.80)
    for at, freq in ((0.0, 587.33), (0.09, 880.00)):
        mix_into(bells, tone(0.70, freq, attack=0.012, decay=0.28, partials=((1.0, 1.0), (2.0, 0.22), (3.0, 0.07)), vibrato=(5.5, 0.003)), at, 0.34)
    mix_into(bells, swell(tone(0.70, 293.66, attack=0.040, decay=0.55, partials=((1.0, 1.0), (1.5, 0.30))), attack=0.08, release=0.30), 0.0, 0.22)
    mix_into(out, reverb(bells, wet=0.28, decay=0.40, tail=0.30, seed=9), 0.02)
    return out


def make_potion() -> Signal:
    """Cork, three gulps, then a rising major arpeggio: drink, then relief."""
    out = buf(1.5)
    # Cork: a bright pop with a fast downward blip, like glass leaving a bottle.
    mix_into(out, envelope(svf(noise(0.06, seed=21), 2400.0, q=2.2), attack=0.0005, decay=0.014), 0.0, 0.40)
    mix_into(out, tone(0.09, 320.0, attack=0.001, decay=0.022, bend=(2.6, 0.012)), 0.0, 0.34)
    # Gulps: descending, lowpassed blips — three of them, unevenly spaced so it
    # sounds like someone drinking rather than a metronome.
    for at, f in ((0.13, 190.0), (0.27, 172.0), (0.40, 158.0)):
        gulp = svf(tone(0.12, f, attack=0.004, decay=0.038, bend=(1.7, 0.030)), 700.0, q=0.8, mode="low")
        mix_into(out, gulp, at, 0.55)
    # Relief: a C major arpeggio walking up to the octave, each note a soft bell.
    arp = buf(1.0)
    for at, f in ((0.0, 523.25), (0.115, 659.25), (0.230, 783.99), (0.345, 1046.50)):
        bell = tone(
            0.85,
            f,
            attack=0.020,
            decay=0.36,
            partials=((1.0, 1.0), (2.0, 0.26), (3.0, 0.09)),
            vibrato=(5.2, 0.0035),
        )
        mix_into(arp, bell, at, 0.30)
    # A low root under the arpeggio: the part that feels like solid ground.
    mix_into(arp, swell(tone(0.85, 130.81, attack=0.06, decay=0.85, partials=((1.0, 1.0), (2.0, 0.3))), attack=0.10, release=0.45), 0.0, 0.34)
    mix_into(out, reverb(arp, wet=0.30, decay=0.45, tail=0.35, seed=2), 0.46)
    # Air: a breath of high noise, felt more than heard.
    mix_into(out, swell(svf(noise(0.8, seed=22), 5200.0, q=0.5, mode="high"), attack=0.25, release=0.5), 0.5, 0.05)
    return out


def make_buff() -> Signal:
    """Upward whoosh into two crystalline chimes: a lasting blessing landing."""
    out = buf(1.1)
    rise = svf(noise(0.45, seed=31), lambda t: 380.0 + 4200.0 * (t / 0.45) ** 1.7, q=0.55)
    mix_into(out, swell(rise, attack=0.30, release=0.10), 0.0, 0.30)
    chimes = buf(1.0)
    for at, f in ((0.0, 659.25), (0.10, 987.77)):
        bell = tone(
            0.85,
            f,
            attack=0.008,
            decay=0.33,
            partials=((1.0, 1.0), (2.76, 0.20), (5.4, 0.06)),
            vibrato=(6.0, 0.003),
        )
        mix_into(chimes, bell, at, 0.34)
    # A held fifth underneath so the buff reads as *lasting*, not as a ping.
    mix_into(chimes, swell(tone(0.85, 329.63, attack=0.05, decay=0.80, partials=((1.0, 1.0), (1.5, 0.35))), attack=0.12, release=0.40), 0.0, 0.22)
    mix_into(out, reverb(chimes, wet=0.34, decay=0.48, tail=0.35, seed=3), 0.30)
    return out


def make_debuff() -> Signal:
    """The mirror of `buff`: downward whoosh into a dark, detuned tritone."""
    out = buf(1.0)
    fall = svf(noise(0.40, seed=41), lambda t: 3200.0 - 2900.0 * (t / 0.40) ** 0.7, q=0.6)
    mix_into(out, swell(fall, attack=0.06, release=0.20), 0.0, 0.30)
    dark = buf(0.9)
    for f, detune in ((196.00, 1.0), (277.18, 0.997)):
        mix_into(
            dark,
            tone(0.75, f * detune, attack=0.010, decay=0.28, partials=((1.0, 1.0), (2.0, 0.30), (3.0, 0.12)), bend=(1.06, 0.25)),
            0.0,
            0.30,
        )
    mix_into(out, reverb(dark, wet=0.32, decay=0.45, tail=0.30, seed=4), 0.24)
    return out


# name -> (renderer, peak level in dBFS, stereo spread in seconds)
#
# Levels are set relative to each other, not just normalized: impacts land
# loudest because they need to cut through, support cues sit back a little so a
# stream of buffs never buries the music, and `critical` sits well below all of
# them because it is only ever heard on top of another sound.
SOUNDS = {
    "strike": (make_strike, -2.5, 0.0),
    "arcane": (make_arcane, -2.5, 0.005),
    "heavy": (make_heavy, -7.5, 0.0),
    "rage": (make_rage, -6.3, 0.0),
    "critical": (make_critical, -8.0, 0.006),
    "heal": (make_heal, -4.5, 0.006),
    "potion": (make_potion, -3.0, 0.006),
    "buff": (make_buff, -4.0, 0.008),
    "debuff": (make_debuff, -4.0, 0.008),
}


def render(name: str, out_dir: str) -> str:
    make, peak_db, spread = SOUNDS[name]
    mono = fade_out(trim_silence(normalize(make(), peak_db)))
    wav_path = os.path.join(tempfile.gettempdir(), f"dx-rpg-{name}.wav")
    write_wav(wav_path, widen(mono, spread))
    ogg_path = os.path.join(out_dir, f"{name}.ogg")
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", wav_path,
         "-c:a", "libvorbis", "-q:a", "4", "-metadata", "artist=dx-rpg", ogg_path],
        check=True,
    )
    os.remove(wav_path)
    print(f"{ogg_path}  {len(mono) / SR:.2f}s  {os.path.getsize(ogg_path)} bytes")
    return ogg_path


def main() -> int:
    default_out = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "assets", "audio", "sfx")
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--out", default=default_out, help="output directory (default: assets/audio/sfx)")
    parser.add_argument("names", nargs="*", choices=[*SOUNDS, []], help="sounds to render (default: all)")
    args = parser.parse_args()

    if shutil.which("ffmpeg") is None:
        print("error: ffmpeg is required to encode Ogg Vorbis", file=sys.stderr)
        return 1
    os.makedirs(args.out, exist_ok=True)
    for name in args.names or SOUNDS:
        render(name, args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

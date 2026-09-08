use dioxus::prelude::*;

use crate::{common::CtxAudioSettings, sfx_cue::Sfx};

const MUSIC_HOME: Asset = asset!("/assets/audio/music/home.ogg");
const MUSIC_OVERWORLD: Asset = asset!("/assets/audio/music/overworld.ogg");

const SFX_HIT: Asset = asset!("/assets/audio/sfx/hit.ogg");
const SFX_CRITICAL: Asset = asset!("/assets/audio/sfx/critical.ogg");
const SFX_DODGE: Asset = asset!("/assets/audio/sfx/dodge.ogg");
const SFX_BLOCK: Asset = asset!("/assets/audio/sfx/block.ogg");
const SFX_HEAL: Asset = asset!("/assets/audio/sfx/heal.ogg");
const SFX_BUFF: Asset = asset!("/assets/audio/sfx/buff.ogg");
const SFX_DEBUFF: Asset = asset!("/assets/audio/sfx/debuff.ogg");
const SFX_POTION: Asset = asset!("/assets/audio/sfx/potion.ogg");
const SFX_VICTORY: Asset = asset!("/assets/audio/sfx/victory.ogg");
const SFX_GAMEOVER: Asset = asset!("/assets/audio/sfx/gameover.ogg");

/// Looping background tracks. Which one (if any) should be playing is decided in
/// `Navbar` from the current `GamePhase` (see its music-transition effect).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicTrack {
    Home,
    Overworld,
}

impl MusicTrack {
    fn asset(self) -> Asset {
        match self {
            MusicTrack::Home => MUSIC_HOME,
            MusicTrack::Overworld => MUSIC_OVERWORLD,
        }
    }
}

fn sfx_asset(sfx: Sfx) -> Asset {
    match sfx {
        Sfx::Hit => SFX_HIT,
        Sfx::CriticalHit => SFX_CRITICAL,
        Sfx::Dodge => SFX_DODGE,
        Sfx::Block => SFX_BLOCK,
        Sfx::Heal => SFX_HEAL,
        Sfx::Buff => SFX_BUFF,
        Sfx::Debuff => SFX_DEBUFF,
        Sfx::Potion => SFX_POTION,
        Sfx::Victory => SFX_VICTORY,
        Sfx::GameOver => SFX_GAMEOVER,
    }
}

/// How far the playback rate of a sound may wander from 1.0, per play.
///
/// The strike sounds fire on nearly every action of every turn, and an identical
/// waveform repeated that often stops reading as an impact and starts reading as
/// a glitch. A few percent of random detune — the standard trick for repeated
/// game impacts — is enough to keep them sounding alive. Stingers that play once
/// per scenario (victory, game over) stay dead-on so they always land the same.
fn sfx_pitch_variation(sfx: Sfx) -> f64 {
    match sfx {
        Sfx::Hit | Sfx::CriticalHit | Sfx::Block | Sfx::Dodge => 0.06,
        Sfx::Heal | Sfx::Buff | Sfx::Debuff | Sfx::Potion => 0.02,
        Sfx::Victory | Sfx::GameOver => 0.0,
    }
}

/// Injects the JS audio bridge once: a persistent looping `<audio>` element for
/// music, plus a `playSfx` helper that fires a fresh `Audio()` per call so
/// overlapping one-shots don't cut each other off. Call once from `App()`, the
/// same way the theme/viewport `document::eval` calls in `main.rs` are — this
/// works uniformly across web, desktop (tao/wry webview), and mobile (Android
/// webview) since all three render through a browser engine.
///
/// Browsers block audio.play() with sound until the page has had a genuine user
/// gesture (click/key/touch) — Home's music auto-starts on mount, before any
/// gesture, so that first play() is silently rejected (some embedded webviews,
/// e.g. VS Code's, are more permissive and don't hit this). A one-time listener
/// below retries as soon as the very first gesture happens anywhere on the page.
///
/// Desktop/mobile-only wrinkle: `dioxus-asset-resolver`'s native protocol handler
/// (what actually serves `asset!()` files in those builds) has no MIME mapping for
/// `.ogg` — or any audio format — at all, and falls back to `Content-Type:
/// text/html` for anything it doesn't recognize. The webview then correctly
/// refuses to play a resource declared as HTML (`NotSupportedError`), regardless
/// of whether the right GStreamer codecs are installed. `loadAsBlobUrl` below
/// fetches the bytes ourselves and wraps them in a `Blob` with an explicit,
/// correct type, bypassing whatever Content-Type the asset server actually sent —
/// harmless overhead on web, where this bug doesn't exist, so no platform-specific
/// branch is needed.
pub fn init_audio_bridge() {
    document::eval(
        r#"
        if (!window.__dxAudio) {
            const bgm = document.createElement('audio');
            bgm.loop = true;
            document.body.appendChild(bgm);
            const describe = (e) => (e && (e.name || e.message)) ? `${e.name}: ${e.message}` : String(e);
            const resumeOnFirstGesture = () => {
                if (bgm.paused && bgm.src) {
                    bgm.play()
                        .then(() => console.debug('[dxAudio] resumed on first gesture'))
                        .catch((e) => console.warn(`[dxAudio] resume on gesture failed: ${describe(e)}`));
                }
                ['pointerdown', 'keydown', 'touchstart'].forEach(
                    (evt) => document.removeEventListener(evt, resumeOnFirstGesture)
                );
            };
            ['pointerdown', 'keydown', 'touchstart'].forEach(
                (evt) => document.addEventListener(evt, resumeOnFirstGesture)
            );
            // One blob URL per asset, kept for the lifetime of the page: sfx fire
            // several times per turn and re-fetching + re-wrapping the same file on
            // every play added audible latency to the first frames of the sound.
            const blobUrls = new Map();
            const loadAsBlobUrl = (src) => {
                let pending = blobUrls.get(src);
                if (!pending) {
                    pending = fetch(src)
                        .then((r) => r.blob())
                        .then((b) => URL.createObjectURL(new Blob([b], { type: 'audio/ogg' })))
                        .catch((e) => {
                            // Don't cache a failure — a later play should retry.
                            blobUrls.delete(src);
                            throw e;
                        });
                    blobUrls.set(src, pending);
                }
                return pending;
            };
            window.__dxAudio = {
                bgm,
                playMusic(src, volume, muted) {
                    bgm.volume = muted ? 0 : volume;
                    if (bgm.dataset.logicalSrc === src) {
                        bgm.play().catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
                        return;
                    }
                    bgm.dataset.logicalSrc = src;
                    loadAsBlobUrl(src).then((url) => {
                        // A newer playMusic call may have already changed the desired
                        // track while this fetch was in flight — don't clobber it.
                        // The blob url itself is owned by `blobUrls`, so it is never
                        // revoked here: switching tracks back and forth reuses it.
                        if (bgm.dataset.logicalSrc !== src) {
                            return;
                        }
                        bgm.src = url;
                        bgm.play().catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
                    }).catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
                },
                stopMusic() {
                    bgm.pause();
                },
                setMusicVolume(volume, muted) {
                    bgm.volume = muted ? 0 : volume;
                },
                playSfx(src, volume, muted, variation) {
                    if (muted || volume <= 0) {
                        return;
                    }
                    loadAsBlobUrl(src).then((url) => {
                        const sfx = new Audio(url);
                        sfx.volume = volume;
                        // Detune this one play a little so the same impact heard turn
                        // after turn doesn't read as a looping sample. Media elements
                        // default to preservesPitch=true, which would resample the
                        // sound to the same pitch and defeat the whole point, so it
                        // has to be turned off first (the prefixed spellings are for
                        // older webviews, which is what the mobile builds run on).
                        if (variation > 0) {
                            sfx.preservesPitch = false;
                            sfx.mozPreservesPitch = false;
                            sfx.webkitPreservesPitch = false;
                            sfx.playbackRate = 1 + (Math.random() * 2 - 1) * variation;
                        }
                        sfx.play().catch((e) => console.warn(`[dxAudio] playSfx failed: ${src}: ${describe(e)}`));
                    }).catch((e) => console.warn(`[dxAudio] playSfx failed: ${src}: ${describe(e)}`));
                },
            };
        }
        "#,
    );
}

/// Starts (or switches to) a looping background track, respecting the current
/// volume/mute settings. `src` values are always compile-time asset paths
/// (`Asset`'s `Display` impl), never user input, so this string-built `eval` call
/// stays injection-safe.
pub fn play_music(track: MusicTrack, settings: CtxAudioSettings) {
    let src = track.asset();
    let volume = settings.music_volume.read().max(0) as f64 / 100.0;
    let muted = *settings.muted.read();
    document::eval(&format!(
        "window.__dxAudio && window.__dxAudio.playMusic('{src}', {volume}, {muted});"
    ));
}

pub fn stop_music() {
    document::eval("window.__dxAudio && window.__dxAudio.stopMusic();");
}

/// Applies a live volume/mute change to the currently-playing track immediately
/// (called from the Navbar's sound-settings controls).
pub fn set_music_volume(settings: CtxAudioSettings) {
    let volume = settings.music_volume.read().max(0) as f64 / 100.0;
    let muted = *settings.muted.read();
    document::eval(&format!(
        "window.__dxAudio && window.__dxAudio.setMusicVolume({volume}, {muted});"
    ));
}

/// Plays a one-shot sound effect. Takes anything that converts into an [`Sfx`],
/// so lib-rpg's `SoundCue` can still be passed straight through at the call sites
/// that already have one.
pub fn play_sfx(sfx: impl Into<Sfx>, settings: CtxAudioSettings) {
    let sfx = sfx.into();
    let src = sfx_asset(sfx);
    let variation = sfx_pitch_variation(sfx);
    let volume = settings.sfx_volume.read().max(0) as f64 / 100.0;
    let muted = *settings.muted.read();
    document::eval(&format!(
        "window.__dxAudio && window.__dxAudio.playSfx('{src}', {volume}, {muted}, {variation});"
    ));
}

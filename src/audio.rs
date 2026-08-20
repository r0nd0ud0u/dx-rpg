use dioxus::prelude::*;
use lib_rpg::common::sound_cue::SoundCue;

use crate::common::CtxAudioSettings;

const MUSIC_HOME: Asset = asset!("/assets/audio/music/home.ogg");
const MUSIC_OVERWORLD: Asset = asset!("/assets/audio/music/overworld.ogg");

const SFX_HIT: Asset = asset!("/assets/audio/sfx/hit.ogg");
const SFX_CRITICAL: Asset = asset!("/assets/audio/sfx/critical.ogg");
const SFX_DODGE: Asset = asset!("/assets/audio/sfx/dodge.ogg");
const SFX_BLOCK: Asset = asset!("/assets/audio/sfx/block.ogg");
const SFX_HEAL: Asset = asset!("/assets/audio/sfx/heal.ogg");

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

fn sfx_asset(cue: SoundCue) -> Asset {
    match cue {
        SoundCue::Hit => SFX_HIT,
        SoundCue::CriticalHit => SFX_CRITICAL,
        SoundCue::Dodge => SFX_DODGE,
        SoundCue::Block => SFX_BLOCK,
        SoundCue::Heal => SFX_HEAL,
    }
}

/// Injects the JS audio bridge once: a persistent looping `<audio>` element for
/// music, plus a `playSfx` helper that fires a fresh `Audio()` per call so
/// overlapping one-shots don't cut each other off. Call once from `App()`, the
/// same way the theme/viewport `document::eval` calls in `main.rs` are — this
/// works uniformly across web, desktop (tao/wry webview), and mobile (Android
/// webview) since all three render through a browser engine.
pub fn init_audio_bridge() {
    document::eval(
        r#"
        if (!window.__dxAudio) {
            const bgm = document.createElement('audio');
            bgm.loop = true;
            document.body.appendChild(bgm);
            window.__dxAudio = {
                bgm,
                playMusic(src, volume, muted) {
                    if (bgm.src.indexOf(src) === -1) {
                        bgm.src = src;
                    }
                    bgm.volume = muted ? 0 : volume;
                    bgm.play().catch(() => {});
                },
                stopMusic() {
                    bgm.pause();
                },
                setMusicVolume(volume, muted) {
                    bgm.volume = muted ? 0 : volume;
                },
                playSfx(src, volume, muted) {
                    if (muted || volume <= 0) {
                        return;
                    }
                    const sfx = new Audio(src);
                    sfx.volume = volume;
                    sfx.play().catch(() => {});
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

/// Plays a one-shot sound effect for the given combat cue.
pub fn play_sfx(cue: SoundCue, settings: CtxAudioSettings) {
    let src = sfx_asset(cue);
    let volume = settings.sfx_volume.read().max(0) as f64 / 100.0;
    let muted = *settings.muted.read();
    document::eval(&format!(
        "window.__dxAudio && window.__dxAudio.playSfx('{src}', {volume}, {muted});"
    ));
}

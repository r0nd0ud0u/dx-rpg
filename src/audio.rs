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

fn sfx_asset(cue: SoundCue) -> Asset {
    match cue {
        SoundCue::Hit => SFX_HIT,
        SoundCue::CriticalHit => SFX_CRITICAL,
        SoundCue::Dodge => SFX_DODGE,
        SoundCue::Block => SFX_BLOCK,
        SoundCue::Heal => SFX_HEAL,
        SoundCue::Potion => SFX_POTION,
        SoundCue::Victory => SFX_VICTORY,
        SoundCue::GameOver => SFX_GAMEOVER,
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
            const loadAsBlobUrl = (src) =>
                fetch(src)
                    .then((r) => r.blob())
                    .then((b) => URL.createObjectURL(new Blob([b], { type: 'audio/ogg' })));
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
                        if (bgm.dataset.logicalSrc !== src) {
                            URL.revokeObjectURL(url);
                            return;
                        }
                        const oldUrl = bgm.src;
                        bgm.src = url;
                        if (oldUrl && oldUrl.startsWith('blob:')) {
                            URL.revokeObjectURL(oldUrl);
                        }
                        bgm.play().catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
                    }).catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
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
                    loadAsBlobUrl(src).then((url) => {
                        const sfx = new Audio(url);
                        sfx.volume = volume;
                        sfx.addEventListener('ended', () => URL.revokeObjectURL(url));
                        sfx.play().catch((e) => {
                            console.warn(`[dxAudio] playSfx failed: ${src}: ${describe(e)}`);
                            URL.revokeObjectURL(url);
                        });
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

/// Plays a one-shot sound effect for the given combat cue.
pub fn play_sfx(cue: SoundCue, settings: CtxAudioSettings) {
    let src = sfx_asset(cue);
    let volume = settings.sfx_volume.read().max(0) as f64 / 100.0;
    let muted = *settings.muted.read();
    document::eval(&format!(
        "window.__dxAudio && window.__dxAudio.playSfx('{src}', {volume}, {muted});"
    ));
}

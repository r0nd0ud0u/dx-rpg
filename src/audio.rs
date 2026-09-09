use dioxus::prelude::*;

use crate::{common::CtxAudioSettings, sfx_cue::Sfx};

const MUSIC_HOME: Asset = asset!("/assets/audio/music/home.ogg");
const MUSIC_OVERWORLD: Asset = asset!("/assets/audio/music/overworld.ogg");

const SFX_STRIKE: Asset = asset!("/assets/audio/sfx/strike.ogg");
const SFX_ARCANE: Asset = asset!("/assets/audio/sfx/arcane.ogg");
const SFX_HEAVY: Asset = asset!("/assets/audio/sfx/heavy.ogg");
const SFX_RAGE: Asset = asset!("/assets/audio/sfx/rage.ogg");
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
        Sfx::Strike => SFX_STRIKE,
        Sfx::Arcane => SFX_ARCANE,
        Sfx::Heavy => SFX_HEAVY,
        Sfx::Rage => SFX_RAGE,
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
    // Prepended rather than interpolated: the script below is full of `{}` and
    // `${}`, none of which would survive being a format string.
    let script = format!(
        "window.__dxPauseOnBlur = {};\n{BRIDGE_JS}",
        cfg!(feature = "mobile")
    );
    document::eval(&script);
}

const BRIDGE_JS: &str = r#"
        if (!window.__dxAudio) {
            const bgm = document.createElement('audio');
            bgm.loop = true;
            document.body.appendChild(bgm);
            const describe = (e) => (e && (e.name || e.message)) ? `${e.name}: ${e.message}` : String(e);
            const PAUSE_ON_BLUR = !!window.__dxPauseOnBlur;
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
            // Strong references to the one-shots currently sounding; see playSfx.
            const playing = new Set();

            // Whether the music is allowed to keep going once the app leaves the
            // screen. Mirrors the player's setting; see `set_background_audio`.
            let backgroundAudio = false;
            // Set only when *we* paused the music because the app went away, so
            // coming back never restarts music that was deliberately stopped —
            // combat silences the track on purpose (see Navbar's music effect).
            let pausedForBackground = false;
            const setAway = (away) => {
                if (away) {
                    if (!backgroundAudio && !bgm.paused) {
                        bgm.pause();
                        pausedForBackground = true;
                        updateMediaSession();
                    }
                } else if (pausedForBackground) {
                    pausedForBackground = false;
                    bgm.play()
                        .then(updateMediaSession)
                        .catch((e) => console.warn(`[dxAudio] resume failed: ${describe(e)}`));
                }
            };
            // The standard signal, and the only one needed on the web: switching tab
            // or locking the phone fires it. `pagehide`/`pageshow` cover being frozen
            // rather than hidden, which is what iOS does.
            document.addEventListener('visibilitychange', () => setAway(document.hidden));
            window.addEventListener('pagehide', () => setAway(true));
            window.addEventListener('pageshow', () => setAway(false));
            // PAUSE_ON_BLUR is set from Rust and is true only on the mobile build:
            // Android's WebView does not reliably deliver `visibilitychange` when the
            // app is backgrounded, so losing window focus is taken as leaving too.
            // Deliberately not done on desktop, where a window blur just means the
            // player clicked something else on the same screen and killing the music
            // for that would be obnoxious.
            if (PAUSE_ON_BLUR) {
                window.addEventListener('blur', () => setAway(true));
                window.addEventListener('focus', () => setAway(false));
            }

            // Asks the host to show transport controls for the music — on Android
            // that is the lock-screen/notification-shade media card, which is what
            // gives the player a way to stop background audio without coming back
            // into the app. Whether it actually appears is up to the embedder:
            // Chrome and Safari honour it, and a plain Android WebView may define
            // the API while never surfacing a notification for it. Harmless where
            // it is ignored.
            const updateMediaSession = () => {
                if (!('mediaSession' in navigator)) {
                    return;
                }
                try {
                    if (typeof MediaMetadata === 'function') {
                        navigator.mediaSession.metadata = new MediaMetadata({ title: 'RPG Adventure' });
                    }
                    navigator.mediaSession.playbackState = bgm.paused ? 'paused' : 'playing';
                    const stop = () => {
                        pausedForBackground = false;
                        bgm.pause();
                        navigator.mediaSession.playbackState = 'paused';
                    };
                    navigator.mediaSession.setActionHandler('pause', stop);
                    navigator.mediaSession.setActionHandler('stop', stop);
                    navigator.mediaSession.setActionHandler('play', () => {
                        bgm.play()
                            .then(() => { navigator.mediaSession.playbackState = 'playing'; })
                            .catch((e) => console.warn(`[dxAudio] play failed: ${describe(e)}`));
                    });
                } catch (e) {
                    console.warn(`[dxAudio] mediaSession unavailable: ${describe(e)}`);
                }
            };
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
                        bgm.play()
                            .then(updateMediaSession)
                            .catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
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
                        bgm.play()
                            .then(updateMediaSession)
                            .catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
                    }).catch((e) => console.warn(`[dxAudio] playMusic failed: ${src}: ${describe(e)}`));
                },
                stopMusic() {
                    // Deliberate, so coming back from the background must not undo it.
                    pausedForBackground = false;
                    bgm.pause();
                    updateMediaSession();
                },

                /// Mirrors the player's "keep playing in the background" setting, and
                /// applies it immediately if the app is already off screen.
                setBackgroundAudio(enabled) {
                    backgroundAudio = enabled;
                    // Applies right away, so turning it off while the app is already
                    // in the background stops the music then and there.
                    setAway(document.hidden);
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
                        // Held until it finishes. Nothing else references a one-shot
                        // once play() has been called, and an element collected
                        // mid-playback is silently cut off — which is exactly what an
                        // intermittently missing sound effect looks like.
                        playing.add(sfx);
                        const release = () => playing.delete(sfx);
                        sfx.addEventListener('ended', release);
                        sfx.addEventListener('error', release);
                        sfx.play().catch((e) => {
                            release();
                            console.warn(`[dxAudio] playSfx failed: ${src}: ${describe(e)}`);
                        });
                    }).catch((e) => console.warn(`[dxAudio] playSfx failed: ${src}: ${describe(e)}`));
                },
            };
        }
        "#;

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

/// Tells the bridge whether the music may keep playing once the app leaves the
/// screen. Applied immediately, so switching it off while the app is already in
/// the background stops the music there and then.
pub fn set_background_audio(settings: CtxAudioSettings) {
    let enabled = *settings.background.read();
    document::eval(&format!(
        "window.__dxAudio && window.__dxAudio.setBackgroundAudio({enabled});"
    ));
}

/// Plays one of the sounds `sfx_cue` picked, exactly as authored: the same attack
/// always sounds identical, with no per-play pitch variation, so a family stays
/// recognisable by ear.
pub fn play_sfx(sfx: Sfx, settings: CtxAudioSettings) {
    let src = sfx_asset(sfx);
    let volume = settings.sfx_volume.read().max(0) as f64 / 100.0;
    let muted = *settings.muted.read();
    document::eval(&format!(
        "window.__dxAudio && window.__dxAudio.playSfx('{src}', {volume}, {muted});"
    ));
}

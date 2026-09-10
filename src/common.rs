use std::path::PathBuf;
#[cfg(feature = "server")]
use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
#[cfg(feature = "server")]
use lib_rpg::server::data_manager::DataManager;

use crate::board_game_components::admin_page::AdminPage;
use crate::board_game_components::create_server_page::CreateServer;
use crate::board_game_components::home_page::Home;
use crate::board_game_components::joinongoinggame_page::JoinOngoingGame;
use crate::board_game_components::lobby_page::LobbyPage;
use crate::board_game_components::navbar::Navbar;
use crate::board_game_components::startgame_page::RunningGamePage;
use colorgrad::{GradientBuilder, LinearGradient};
use once_cell::sync::Lazy;

// Global signals
pub static SERVER_NAME: GlobalSignal<String> = Signal::global(String::new);

/// server only: Data manager
#[cfg(feature = "server")]
pub static DATA_MANAGER: Lazy<Arc<Mutex<DataManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(DataManager::default())));

// Lazy
pub static ADMIN: Lazy<String> = Lazy::new(|| "Admin".to_owned());
pub static DISCONNECTED_USER: Lazy<String> = Lazy::new(|| "not connected".to_owned());
pub static SAVED_DATA: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("saved_data"));

pub static ENERGY_GRAD: Lazy<LinearGradient> = Lazy::new(|| {
    GradientBuilder::new()
        .html_colors(&["#ff2600ff", "#f2ff00ff", "#11c426ff"])
        .build::<colorgrad::LinearGradient>()
        .expect("Failed to build gradient")
});

#[derive(Debug, Clone, Routable, PartialEq, serde::Serialize, serde::Deserialize,)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/admin-page")]
    AdminPage {},
    #[route("/create-server")]
    CreateServer {},
    #[route("/lobby-page")]
    LobbyPage {},
    #[route("/running-game")]
    RunningGamePage {},
    #[route("/current-game")]
    JoinOngoingGame {},
}

pub const PATH_IMG: Asset = asset!("/assets/img");
pub const DX_COMP_CSS: Asset = asset!("/assets/dx-components-theme.css");
/// Bundled Inter subsets — see `inter_font_face_css` and `assets/fonts/Sources.txt`.
pub const PATH_FONTS: Asset = asset!("/assets/fonts");

/// The UI font stack. Inter first (bundled, see `inter_font_face_css`), then the system
/// UI faces it most resembles, so text laid out before the face is ready — or if the
/// asset ever fails to load — still looks like the app rather than like Times New Roman.
pub const FONT_STACK: &str =
    "'Inter', 'Segoe UI', Roboto, 'Helvetica Neue', Arial, system-ui, sans-serif";

/// Background and text of the very first paint, before any stylesheet applies. Kept in
/// sync by hand with `--rpg-bg` / `--rpg-text` in `assets/dx-components-theme.css`; they
/// are literals here because this string is what the webview shows *before* the token
/// definitions are parsed.
pub const BOOT_BG: &str = "#080c14";
pub const BOOT_TEXT: &str = "#e2e8f0";

/// `BOOT_BG` again, as the webview wants it: the colour wry paints before it has a
/// document at all, on both native clients (`main.rs`). A test keeps the two in step.
pub const BOOT_BG_RGBA: (u8, u8, u8, u8) = (0x08, 0x0c, 0x14, 0xff);

/// `@font-face` rules for the bundled Inter files.
///
/// Built in Rust rather than written into a stylesheet because the asset pipeline hashes
/// the folder name (`fonts-dxh…`, like `img-dxh…`), so a hand-written `url()` inside a CSS
/// file would not resolve — the same reason sprites are addressed through `PATH_IMG`.
///
/// Two subsets per style: `latin` covers English, `latin-ext` the accented characters the
/// French locale and the LOTR names need. `swap` rather than `block` so text is never
/// invisible; the files are local, so the swap is a frame at most.
pub fn inter_font_face_css() -> String {
    // Ranges as published with the Google Fonts subsets these files came from.
    const LATIN: &str = "U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD";
    const LATIN_EXT: &str = "U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF, U+0304, U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020, U+20A0-20AB, U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF";
    [
        ("normal", "latin", LATIN),
        ("normal", "latin-ext", LATIN_EXT),
        ("italic", "latin", LATIN),
        ("italic", "latin-ext", LATIN_EXT),
    ]
    .iter()
    .map(|(style, subset, range)| {
        format!(
            "@font-face{{font-family:'Inter';font-style:{style};font-weight:100 900;\
             font-display:swap;src:url('{PATH_FONTS}/inter-{subset}-{style}.woff2') format('woff2');\
             unicode-range:{range};}}"
        )
    })
    .collect()
}

/// The handful of rules that decide what the window looks like in the moment between
/// "webview has a document" and "the app's stylesheets are applied": the dark ground, the
/// text colour, no default body margin, and the font stack. Inlined into the desktop
/// index's `<head>` (see `main.rs`) so it needs no request of its own.
pub fn boot_critical_css() -> String {
    format!(
        "html,body{{background-color:{BOOT_BG};color:{BOOT_TEXT};margin:0;\
         font-family:{FONT_STACK};}}"
    )
}

pub const OFFLINE_PATH: &str = "offlines";

/// Native clients only: `dioxus_sdk_storage` keys for the user-chosen server address
/// and TLS-validation override, shared between `main.rs` (reads them once at startup,
/// before launch) and `board_game_components/navbar.rs` (the settings UI that writes
/// them). Kept as constants so the two call sites can't drift apart.
pub const SYNCED_SERVER_URL_KEY: &str = "synced_server_url";
pub const SYNCED_INSECURE_CERTS_KEY: &str = "synced_insecure_certs";

/// Per-device/browser random token, generated once and persisted locally (both web and
/// native). Sent alongside `LoginAllSessions`/`AddPlayer` so the server can tell a genuine
/// reconnect of the same device apart from a different device that still has a stale cached
/// session for the same username (see `websocket_handler::event::add_player`).
pub const SYNCED_DEVICE_TOKEN_KEY: &str = "synced_device_token";

/// Audio settings, persisted locally (both web and native) and shared between
/// `main.rs` (declares them via `use_synced_storage`) and `board_game_components/navbar.rs`
/// / `audio.rs` (read/write them to control playback).
pub const SYNCED_MUSIC_VOLUME_KEY: &str = "synced_music_volume";
pub const SYNCED_SFX_VOLUME_KEY: &str = "synced_sfx_volume";
pub const SYNCED_AUDIO_MUTED_KEY: &str = "synced_audio_muted";
/// Whether music keeps playing once the app leaves the screen. Off by default (battery),
/// but some players want it, hence a setting.
pub const SYNCED_BACKGROUND_AUDIO_KEY: &str = "synced_background_audio";

/// Overworld map zoom. Device-local, not per-account: the right zoom depends on the
/// screen, and this also works offline, where there is no server to store settings on.
pub const SYNCED_OVERWORLD_ZOOM_KEY: &str = "synced_overworld_zoom";

/// Whether this device has already been shown the how-to-play dialog. Device-local
/// rather than per-account: the point is that a first-time player gets the tutorial
/// without asking for it, and that includes the offline mode, where there is no account.
pub const SYNCED_TUTORIAL_SEEN_KEY: &str = "synced_tutorial_seen";

/// Whether the player has dismissed the first-scenario combat hints. Device-local for the
/// same reasons as `SYNCED_TUTORIAL_SEEN_KEY`; without it a veteran would be coached again
/// at the start of every new run.
pub const SYNCED_COMBAT_HINTS_OFF_KEY: &str = "synced_combat_hints_off";

// ── Per-setting context newtypes ─────────────────────────────────────────────
// Each wraps a `Signal<bool>` in a distinct type so that Dioxus context lookup
// (which is keyed by TypeId) stores and retrieves them independently.

/// Whether attack animations are enabled on the board
#[derive(Clone, Copy)]
pub struct CtxToggleAtkAnimation(pub Signal<bool>);

/// Whether boss energy (mana/vigor/berserk) bars are shown
#[derive(Clone, Copy)]
pub struct CtxShowBossEnergy(pub Signal<bool>);

/// Whether hero aggro values are shown on character panels
#[derive(Clone, Copy)]
pub struct CtxShowHeroAggro(pub Signal<bool>);

/// Whether attack tooltip descriptions are shown on hover
#[derive(Clone, Copy)]
pub struct CtxShowAtkTooltips(pub Signal<bool>);

/// Whether the boss HP bar is shown
#[derive(Clone, Copy)]
pub struct CtxShowBossHp(pub Signal<bool>);

/// Whether an auto-save should be triggered at the start of each scenario
#[derive(Clone, Copy)]
pub struct CtxAutoSaveScenario(pub Signal<bool>);

/// Whether the Store is accessible during an active scenario (default: off)
#[derive(Clone, Copy)]
pub struct CtxShopEnabled(pub Signal<bool>);

/// Per-character attack panel order, keyed by `id_name`. Loaded on demand from
/// `user_settings` (`atk_panel_order_key` in `character_page.rs`), not eagerly like the
/// `CtxShow*` toggles — the key space is per character, not a fixed handful.
#[derive(Clone, Copy)]
pub struct CtxAtkPanelOrders(pub Signal<std::collections::HashMap<String, Vec<String>>>);

/// Current UI language ("en" or "fr"), synced to browser localStorage (works
/// pre-login, unlike the SQLite-backed CtxShow* settings above). Drives both
/// dioxus-i18n's t!() chrome strings (via the sync effect in main.rs) and
/// lib-rpg's Lang resolver for bilingual descriptions (see lang_from_app_lang).
#[derive(Clone, Copy)]
pub struct CtxAppLang(pub Signal<String>);

/// Native only: user-chosen server address, applied on the next launch (see `main.rs`).
/// Declared in `App()`, not Navbar — `use_synced_storage` in a `#[layout]` component
/// stack-overflows at startup.
#[derive(Clone, Copy)]
pub struct CtxSyncedServerUrl(pub Signal<String>);

/// Native only: accept self-signed TLS certs for the server above. Same constraints as
/// `CtxSyncedServerUrl`.
#[derive(Clone, Copy)]
pub struct CtxSyncedInsecureCerts(pub Signal<bool>);

/// This device/browser's persistent random token — see `SYNCED_DEVICE_TOKEN_KEY`. Provided
/// via context (declared in `App()`) for the same reasons as `CtxSyncedServerUrl`.
#[derive(Clone, Copy)]
pub struct CtxDeviceToken(pub Signal<String>);

/// Whether the how-to-play dialog has already been shown on this device — see
/// `SYNCED_TUTORIAL_SEEN_KEY`. Declared in `App()` for the same reason as
/// `CtxSyncedServerUrl`, and flipped by `Navbar` when the dialog closes.
#[derive(Clone, Copy)]
pub struct CtxTutorialSeen(pub Signal<bool>);

/// Whether the first-scenario combat hints have been dismissed — see
/// `SYNCED_COMBAT_HINTS_OFF_KEY`. Declared in `App()`, read by `GameBoard` and written by
/// the hint bar's ✕ (`board_game_components/tutorial.rs`).
#[derive(Clone, Copy)]
pub struct CtxCombatHintsOff(pub Signal<bool>);

/// Overworld zoom as a scale factor (`1.0` = 100%), via `SYNCED_OVERWORLD_ZOOM_KEY`.
/// Declared in `App()`, not `OverworldMap`: the map is unmounted for the whole of a
/// fight, so anything it loads itself restarts from the default on every remount.
#[derive(Clone, Copy)]
pub struct CtxOverworldZoom(pub Signal<f32>);

/// Audio playback settings — music volume, sfx volume (each 0-100), and a mute-all
/// toggle. Persisted via `SYNCED_MUSIC_VOLUME_KEY`/`SYNCED_SFX_VOLUME_KEY`/
/// `SYNCED_AUDIO_MUTED_KEY`, declared in `App()`, and read by `audio.rs`'s
/// `play_music`/`play_sfx` plus the volume controls in `Navbar`.
#[derive(Clone, Copy)]
pub struct CtxAudioSettings {
    pub music_volume: Signal<i32>,
    pub sfx_volume: Signal<i32>,
    pub muted: Signal<bool>,
    /// See `SYNCED_BACKGROUND_AUDIO_KEY`.
    pub background: Signal<bool>,
}

/// State of the client's websocket link to the server, tracked by the reconnect loop in
/// `main.rs`'s ws-loop `use_future` and surfaced in `Navbar` as a small status badge —
/// on a flaky desktop/mobile connection the game otherwise just silently stops
/// responding, with no indication of whether that's a bug or the network.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Reconnecting,
}

/// See [`ConnectionStatus`]. Declared in `App()`, same reasoning as the other `Ctx*`
/// context newtypes above.
#[derive(Clone, Copy)]
pub struct CtxConnectionStatus(pub Signal<ConnectionStatus>);

/// Set when the server session no longer backs the identity the client believes it holds
/// (see `AdminPage`'s check) — typically idled out past `SessionConfig`'s lifespan on a
/// device never signed out of. `LoginPage` shows a "session expired" banner until dismissed
/// or a new login.
#[derive(Clone, Copy)]
pub struct CtxSessionExpired(pub Signal<bool>);

/// Round-trip latency in ms from `main.rs`'s ping loop. `None` means no measurement yet or
/// the last one timed out — a congested link stays open and `Connected` while pings stop
/// arriving. Rendered in `Navbar` as a filling wifi icon.
#[derive(Clone, Copy)]
pub struct CtxConnectionLatency(pub Signal<Option<u64>>);

/// Converts the app's "en"/"fr" locale string into lib-rpg's `Lang` enum —
/// the one conversion boundary between the two crates' locale representations
/// (lib-rpg has no dioxus/unic_langid dependency).
pub fn lang_from_app_lang(app_lang: &str) -> lib_rpg::common::lang::Lang {
    if app_lang == "fr" {
        lib_rpg::common::lang::Lang::Fr
    } else {
        lib_rpg::common::lang::Lang::En
    }
}

/// URL for a character photo. `.png` is appended when `photo_name` has no extension
/// (legacy entries stored only the stem).
///
/// Native clients aren't same-origin with the game server — their webview loads from a
/// local asset protocol — so the root-relative path needs the server base prefixed.
/// `get_server_url()` is `""` on web, so prefixing only when non-empty covers both.
pub fn photo_src(photo_name: &str) -> String {
    let path = if photo_name.contains('.') {
        format!("/img-srv/{}", photo_name)
    } else {
        format!("/img-srv/{}.png", photo_name)
    };
    let base = dioxus::fullstack::get_server_url();
    if base.is_empty() {
        path
    } else {
        format!("{base}{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::{lang_from_app_lang, photo_src};
    use lib_rpg::common::lang::Lang;

    #[test]
    fn font_face_css_is_one_rule_per_subset_and_style() {
        let css = super::inter_font_face_css();
        assert_eq!(4, css.matches("@font-face").count());
        for file in [
            "inter-latin-normal.woff2",
            "inter-latin-ext-normal.woff2",
            "inter-latin-italic.woff2",
            "inter-latin-ext-italic.woff2",
        ] {
            assert!(css.contains(file), "missing {file} in {css}");
        }
        // The `\`-continued format strings must not leak newlines into the CSS.
        assert!(
            !css.contains('\n'),
            "font-face css must stay on one line: {css}"
        );
    }

    #[test]
    fn boot_background_is_the_same_colour_in_both_forms() {
        let (r, g, b, a) = super::BOOT_BG_RGBA;
        assert_eq!(super::BOOT_BG, format!("#{r:02x}{g:02x}{b:02x}"));
        assert_eq!(0xff, a, "the window background must be opaque");
    }

    #[test]
    fn boot_css_paints_the_dark_ground_before_any_stylesheet() {
        let css = super::boot_critical_css();
        assert!(css.contains(super::BOOT_BG));
        assert!(css.contains("margin:0"));
        assert!(!css.contains('\n'), "boot css must stay on one line: {css}");
    }

    #[test]
    fn unit_lang_from_app_lang() {
        assert_eq!(lang_from_app_lang("fr"), Lang::Fr);
        assert_eq!(lang_from_app_lang("en"), Lang::En);
        // Anything unrecognized defaults to English.
        assert_eq!(lang_from_app_lang(""), Lang::En);
        assert_eq!(lang_from_app_lang("fr-FR"), Lang::En);
    }

    #[test]
    fn unit_photo_src_appends_png_extension_when_missing() {
        // No test ever calls `set_server_url()`, so `get_server_url()` stays
        // "" here — same as on web, which defaults to same-origin.
        assert_eq!(photo_src("hero_stem"), "/img-srv/hero_stem.png");
    }

    #[test]
    fn unit_photo_src_keeps_existing_extension() {
        assert_eq!(photo_src("hero.jpg"), "/img-srv/hero.jpg");
    }
}

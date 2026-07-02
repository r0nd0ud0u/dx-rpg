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
use crate::board_game_components::loadgame_page::LoadGame;
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
    #[route("/load-game")]
    LoadGame {},
    #[route("/current-game")]
    JoinOngoingGame {},
}

pub const PATH_IMG: Asset = asset!("/assets/img");
pub const DX_COMP_CSS: Asset = asset!("/assets/dx-components-theme.css");

pub const OFFLINE_PATH: &str = "offlines";

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

/// Current UI language ("en" or "fr"), synced to browser localStorage (works
/// pre-login, unlike the SQLite-backed CtxShow* settings above). Drives both
/// dioxus-i18n's t!() chrome strings (via the sync effect in main.rs) and
/// lib-rpg's Lang resolver for bilingual descriptions (see lang_from_app_lang).
#[derive(Clone, Copy)]
pub struct CtxAppLang(pub Signal<String>);

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

/// Returns the URL for serving a character photo via the dynamic image route.
/// If `photo_name` already contains an extension (has a dot), the URL is used
/// as-is; otherwise `.png` is appended for backward-compat with legacy entries
/// that stored only the filename stem.
pub fn photo_src(photo_name: &str) -> String {
    if photo_name.contains('.') {
        format!("/img-srv/{}", photo_name)
    } else {
        format!("/img-srv/{}.png", photo_name)
    }
}

#[cfg(test)]
mod tests {
    use super::lang_from_app_lang;
    use lib_rpg::common::lang::Lang;

    #[test]
    fn unit_lang_from_app_lang() {
        assert_eq!(lang_from_app_lang("fr"), Lang::Fr);
        assert_eq!(lang_from_app_lang("en"), Lang::En);
        // Anything unrecognized defaults to English.
        assert_eq!(lang_from_app_lang(""), Lang::En);
        assert_eq!(lang_from_app_lang("fr-FR"), Lang::En);
    }
}

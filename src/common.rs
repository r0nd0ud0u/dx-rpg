use dioxus::prelude::*;

use crate::board_game_components::admin_page::AdminPage;
use crate::board_game_components::create_server_page::CreateServer;
use crate::board_game_components::home_page::Home;
use crate::board_game_components::joinongoinggame_page::JoinOngoingGame;
use crate::board_game_components::loadgame_page::LoadGame;
use crate::board_game_components::lobby_page::LobbyPage;
use crate::board_game_components::navbar::Navbar;
use crate::board_game_components::startgame_page::StartGamePage;
use colorgrad::{GradientBuilder, LinearGradient};
use once_cell::sync::Lazy;

pub static SERVER_NAME: GlobalSignal<String> = Signal::global(String::new);

pub fn disconnected_user() -> String {
    "not connected".to_owned()
}
pub static ADMIN: Lazy<String> = Lazy::new(|| "Admin".to_string());

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
    #[route("/start-game")]
    StartGamePage {},
    #[route("/load-game")]
    LoadGame {},
    #[route("/current-game")]
    JoinOngoingGame {},
}

pub const PATH_IMG: Asset = asset!("/assets/img");
pub const DX_COMP_CSS: Asset = asset!("/assets/dx-components-theme.css");
pub const SAVED_GAME_MANAGER: &str = "game_manager.json";
pub const SAVED_GAME_MANAGER_REPLAY: &str = "game_manager_replay.json";

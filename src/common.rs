use dioxus::prelude::*;

use crate::application::Application;
use crate::components::create_server_page::CreateServer;
use crate::components::home_page::Home;
use crate::components::joinongoinggame_page::JoinOngoingGame;
use crate::components::loadgame_page::LoadGame;
use crate::components::lobby_page::LobbyPage;
use crate::components::navbar::Navbar;
use crate::components::startgame_page::StartGamePage;
use colorgrad::{CatmullRomGradient, GradientBuilder};
use once_cell::sync::Lazy;

pub static APP: GlobalSignal<Application> = Signal::global(Application::default);

pub static ENERGY_GRAD: Lazy<CatmullRomGradient> = Lazy::new(|| {
    GradientBuilder::new()
        .html_colors(&["deeppink", "gold", "seagreen"])
        .build::<CatmullRomGradient>()
        .expect("Failed to build gradient")
});

pub mod tempo_const {
    pub const AUTO_ATK_TEMPO_MS: u64 = 3000;
    pub const TIMER_FUTURE_1S: u64 = 1000;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonStatus {
    StartGame = 0,
    ReplayGame,
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
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

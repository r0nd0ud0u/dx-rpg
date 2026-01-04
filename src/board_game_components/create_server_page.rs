use dioxus::prelude::*;

use crate::{board_game_components::common_comp::ButtonLink, common::Route};

/// CreateServer page
#[component]
pub fn CreateServer() -> Element {
    rsx! {
        div { class: "home-container",
            ButtonLink {
                target: Route::LobbyPage {}.into(),
                name: "New Game".to_string(),
            }
            ButtonLink {
                target: Route::LoadGame {}.into(),
                name: "Load Game".to_string(),
            }
        }
    }
}

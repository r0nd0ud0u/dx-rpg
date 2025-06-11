use dioxus::prelude::*;

use crate::{common::Route, components::common_comp::ButtonLink};

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

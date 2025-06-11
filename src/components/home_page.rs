use dioxus::prelude::*;

use crate::{common::Route, components::common_comp::ButtonLink};

/// Home page
#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "home-container",
            h1 { "Welcome to the RPG game!" }
            ButtonLink {
                target: Route::CreateServer {}.into(),
                name: "Create Server".to_string(),
            }
            ButtonLink {
                target: Route::JoinOngoingGame {}.into(),
                name: "Join game".to_string(),
            }
        }
    }
}

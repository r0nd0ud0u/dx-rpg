use dioxus::prelude::*;

use crate::{
    board_game_components::common_comp::ButtonLink,
    common::Route,
    components::button::{Button, ButtonVariant},
};

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
        Demo {}
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { display: "flex", flex_direction: "column", gap: "0.5rem",
            Button { "Primary" }

            Button { variant: ButtonVariant::Secondary, "Secondary" }

            Button { variant: ButtonVariant::Destructive, "Destructive" }

            Button { variant: ButtonVariant::Outline, "Outline" }

            Button { variant: ButtonVariant::Ghost, "Ghost" }
        }
    }
}

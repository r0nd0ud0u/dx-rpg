use dioxus::prelude::*;

use crate::{
    board_game_components::{common_comp::ButtonLink, login_page::LoginPage},
    common::{disconnected_user, Route, USER_NAME},
};

/// Home page
#[component]
pub fn Home() -> Element {
    if USER_NAME == disconnected_user() {
        rsx! {
            LoginPage {}
        }
    } else {
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
}

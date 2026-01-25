use dioxus::prelude::*;

use crate::{
    board_game_components::{common_comp::ButtonLink, login_page::LoginPage},
    common::{Route, USER_NAME, disconnected_user},
    websocket_handler::game_state::GameStateWebsocket,
};

/// Home page
#[component]
pub fn Home() -> Element {
    let gsw_sig = use_context::<Signal<GameStateWebsocket>>();
    // Snapshot for this render
    let gsw = gsw_sig();

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
                for player in gsw.players {
                    p { "{player}" }
                }
            }
        }
    }
}

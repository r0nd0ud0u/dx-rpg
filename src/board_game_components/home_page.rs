use dioxus::prelude::*;

use crate::{
    board_game_components::{common_comp::ButtonLink, login_page::LoginPage},
    common::{Route, disconnected_user},
    websocket_handler::game_state::GameStateWebsocket,
};

/// Home page
#[component]
pub fn Home() -> Element {
    // contexts
    let gsw_sig = use_context::<Signal<GameStateWebsocket>>();
    let local_login_session = use_context::<Signal<String>>();
    // Snapshot for this render
    let gsw = gsw_sig();
    let user_name = local_login_session();

    if user_name == disconnected_user() {
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

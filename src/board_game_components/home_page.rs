use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};

use crate::{
    board_game_components::{common_comp::ButtonLink, login_page::LoginPage},
    common::{Route, disconnected_user},
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        game_state::{GamePhase, ServerData},
    },
};

/// Home page
#[component]
pub fn Home() -> Element {
    // contexts
    let local_login_name_session = use_context::<Signal<String>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>;
    // Snapshot for this render
    let user_name = local_login_name_session();

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
                    onclick: move |_| {
                        server_data().write().app.game_phase = GamePhase::Default;
                    },
                }
                ButtonLink {
                    target: Route::JoinOngoingGame {}.into(),
                    name: "Join game".to_string(),
                    onclick: move |_| {
                        async move {
                            let _ = socket.send(ClientEvent::RequestOnGoingGamesList).await;
                        }
                    },
                }
            }
        }
    }
}

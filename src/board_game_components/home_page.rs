use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::{
    board_game_components::{common_comp::ButtonLink, login_page::LoginPage},
    common::{DISCONNECTED_USER, Route},
    websocket_handler::event::{ClientEvent, ServerEvent},
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

    if user_name == *DISCONNECTED_USER {
        rsx! {
            LoginPage {}
        }
    } else {
        rsx! {
            div { class: "home-container",
                div { class: "rotate-scale-up",
                    h1 { "Welcome to the RPG game!" }
                }
                ButtonLink {
                    target: Route::CreateServer {}.into(),
                    name: "Create Server".to_string(),
                    onclick: move |_| {
                        server_data().write().core_game_data.game_phase = GamePhase::Default;
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

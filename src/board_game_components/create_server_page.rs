use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};

use crate::{
    common::Route,
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::{request_update_saved_game_list_display, send_initialize_game},
    },
};

/// CreateServer page
#[component]
pub fn CreateServer() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();
    rsx! {
        div { class: "home-container",
            h1 { class: "rpg-title", "🏰 Server" }
            p { class: "rpg-subtitle", "Choose your adventure" }
            div { class: "action-grid",
                div { class: "action-card",
                    span { class: "action-icon", "🆕" }
                    Link {
                        class: "header-text",
                        to: Route::LobbyPage {},
                        onclick: move |_| {
                            let user_name = local_login_name_session();
                            async move {
                                send_initialize_game(&user_name, socket).await;
                            }
                        },
                        "New Game"
                    }
                    p { class: "action-desc", "Start a fresh campaign" }
                }
                div { class: "action-card",
                    span { class: "action-icon", "💾" }
                    Link {
                        class: "header-text",
                        to: Route::LoadGame {},
                        onclick: move |_| {
                            let user_name = local_login_name_session();
                            async move {
                                request_update_saved_game_list_display(socket, &user_name).await;
                            }
                        },
                        "Load Game"
                    }
                    p { class: "action-desc", "Continue a saved adventure" }
                }
            }
        }
    }
}

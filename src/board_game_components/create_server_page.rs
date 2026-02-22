use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};

use crate::{
    board_game_components::{
        common_comp::ButtonLink,
        msg_from_client::{request_update_saved_game_list_display, send_initialize_game},
    },
    common::Route,
    websocket_handler::event::{ClientEvent, ServerEvent},
};

/// CreateServer page
#[component]
pub fn CreateServer() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();
    rsx! {
        div { class: "home-container",
            h1 { "Create a server" }
            ButtonLink {
                target: Route::LobbyPage {}.into(),
                name: "New Game",
                onclick: move |_| {
                    let user_name = local_login_name_session();
                    async move {
                        send_initialize_game(&user_name, socket).await;
                    }
                },
            }
            ButtonLink {
                target: Route::LoadGame {}.into(),
                name: "Load Game",
                onclick: move |_| {
                    let user_name = local_login_name_session();
                    async move {
                        request_update_saved_game_list_display(socket, &user_name).await;
                    }
                },
            }
        }
    }
}

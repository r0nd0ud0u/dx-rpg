use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};

use crate::{
    board_game_components::{
        msg_from_client::{request_update_saved_game_list_display, send_initialize_game},
    },
    common::Route,
    components::button::Button,
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
            Button {
                onclick: move |_| {
                    let user_name = local_login_name_session();
                    async move {
                        send_initialize_game(&user_name, socket).await;
                        let navigator = use_navigator();
                        navigator.push(Route::LobbyPage {});
                    }
                },
                "New Game"
            }
            Button {
                onclick: move |_| {
                    async move {
                        request_update_saved_game_list_display(socket).await;
                        let navigator = use_navigator();
                        navigator.push(Route::LoadGame {});
                    }
                },
                "Load Game"
            }
        }
    }
}

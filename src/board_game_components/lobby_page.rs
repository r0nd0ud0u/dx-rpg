use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};

use crate::{
    application::Application,
    auth_manager::server_fn::get_user_name,
    board_game_components::{common_comp::ButtonLink, msg_from_client::send_start_game},
    common::{Route, SERVER_NAME},
    websocket_handler::event::{ClientEvent, ServerEvent},
};

#[component]
pub fn LobbyPage() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let app = use_context::<Signal<Application>>();

    rsx! {
        div { class: "home-container",
            h1 { "LobbyPage" }
            if app.read().is_game_running {
                ButtonLink {
                    target: Route::StartGamePage {}.into(),
                    name: "Start Game".to_string(),
                    onclick: move |_| async move {
                        send_start_game(socket.clone()).await;
                    },
                }
            }
        }
    }
}

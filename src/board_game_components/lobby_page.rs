use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};

use crate::{
    application::Application,
    board_game_components::{
        character_select::CharacterSelect, common_comp::ButtonLink,
        msg_from_client::send_start_game,
    },
    common::Route,
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        game_state::ServerData,
    },
};

#[component]
pub fn LobbyPage() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();

    rsx! {
        div { class: "home-container",
            h1 { "LobbyPage" }

            if server_data.read().app.is_game_running {
                ButtonLink {
                    target: Route::StartGamePage {}.into(),
                    name: "Start Game".to_string(),
                    onclick: move |_| async move {
                        send_start_game(socket).await;
                    },
                }
                CharacterSelect {}
            }
        
        }

    }
}

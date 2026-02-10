use dioxus::logger::tracing;
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};

use crate::{
    board_game_components::{
        character_select::CharacterSelect, common_comp::ButtonLink,
        msg_from_client::send_start_game, startgame_page::StartGamePage,
    },
    common::{Route, SERVER_NAME},
    components::button::Button,
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        game_state::{GamePhase, ServerData},
    },
};

#[component]
pub fn LobbyPage() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let game_phase = use_context::<Signal<GamePhase>>();

    rsx! {
        // if the game is not running, show the lobby page, otherwise show the start game page
        if game_phase() == GamePhase::InitGame {
            div { class: "home-container",
                h1 { "LobbyPage" }
                // if the current client is the host, show start game button
                if SERVER_NAME() == local_login_name_session() {
                    Button {
                        onclick: move |_| async move {
                            send_start_game(socket).await;
                        },
                        "Start Game"
                    }
                }
                // show character select page
                CharacterSelect {}
            }
        } else if game_phase() == GamePhase::Running {
            StartGamePage {}
        } else if game_phase() == GamePhase::Ended {
            ButtonLink {
                target: Route::Home {}.into(),
                name: "No more game, back to Home".to_string(),
            }
        }
    }
}

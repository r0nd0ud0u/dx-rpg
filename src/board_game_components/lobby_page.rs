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
    let local_login_name_session = use_context::<Signal<String>>();
    let server_data = use_context::<Signal<ServerData>>();

    // all players info have a character name
    let server_data_snap = server_data();

    let all_players_have_character_name = if server_data_snap.players_info.is_empty() {
        false
    } else {
        server_data_snap
            .players_info
            .values()
            .all(|p| !p.character_names.is_empty())
    };
    tracing::info!(
        "all_players_have_character_name: {}",
        all_players_have_character_name
    );

    rsx! {
        // if the game is not running, show the lobby page, otherwise show the start game page
        if server_data_snap.app.game_phase == GamePhase::InitGame
            || server_data_snap.app.game_phase == GamePhase::Loading
        {
            div { class: "home-container",
                if server_data_snap.app.game_phase == GamePhase::InitGame {
                    h1 { "Init game" }
                } else if server_data_snap.app.game_phase == GamePhase::Loading {
                    h1 { "Loading" }
                } else {
                    h1 { "Lobby page" }
                }
                // if the current client is the host, show start game button
                if SERVER_NAME() == local_login_name_session() && all_players_have_character_name
                    && (server_data_snap.app.game_phase == GamePhase::InitGame
                        || server_data_snap.app.game_phase == GamePhase::Loading
                            && server_data_snap.app.players_nb
                                == server_data_snap.players_info.len() as i64)
                {
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
        } else if server_data_snap.app.game_phase == GamePhase::Running {
            // check if there is more characters in game than users
            if server_data_snap.app.game_manager.pm.active_heroes.len()
                <= server_data_snap.players_info.len()
            {
                StartGamePage {}
            } else {

                ButtonLink {
                    target: Route::Home {}.into(),
                    name: "Not enough players".to_string(),
                    onclick: move |_| {
                        async move {
                            let _ = socket
                                .send(
                                    ClientEvent::DisconnectFromServerData(
                                        SERVER_NAME(),
                                        local_login_name_session(),
                                    ),
                                )
                                .await;
                        }
                    },
                }
            }
        } else if server_data_snap.app.game_phase == GamePhase::Ended {
            ButtonLink {
                target: Route::Home {}.into(),
                name: "No more game, back to home".to_string(),
            }
        } else if server_data_snap.app.game_phase == GamePhase::Default {
            // nothing to display
        }
    }
}

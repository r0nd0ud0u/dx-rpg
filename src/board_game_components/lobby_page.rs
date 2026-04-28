use dioxus::logger::tracing;
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::components::button::ButtonVariant;
use crate::{
    board_game_components::{
        character_select::CharacterSelect, common_comp::ButtonLink, startgame_page::RunningGamePage,
    },
    common::{Route, SERVER_NAME},
    components::button::Button,
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::send_start_game,
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

    let all_players_have_character_name = if server_data_snap.players_data.players_info.is_empty() {
        false
    } else {
        server_data_snap
            .players_data
            .players_info
            .values()
            .all(|p| !p.character_id_names.is_empty())
    };
    tracing::trace!(
        "all_players_have_character_name: {}",
        all_players_have_character_name
    );

    rsx! {
        // if the game is not running, show the lobby page, otherwise show the start game page
        if server_data_snap.core_game_data.game_phase == GamePhase::InitGame
            || server_data_snap.core_game_data.game_phase == GamePhase::Loading
        {
            div { class: "lobby-page",
                h2 { class: "rpg-title",
                    if server_data_snap.core_game_data.game_phase == GamePhase::InitGame {
                        "⚔️ Lobby"
                    } else {
                        "⏳ Loading…"
                    }
                }
                div { class: "lobby-info-bar",
                    div { class: "lobby-info-item",
                        span { class: "lobby-info-label", "Server" }
                        span { class: "lobby-info-value", "{SERVER_NAME()}" }
                    }
                    div { class: "lobby-info-item",
                        span { class: "lobby-info-label", "Players" }
                        span { class: "lobby-info-value",
                            "{server_data_snap.players_data.players_info.len()} / {server_data_snap.core_game_data.players_nb}"
                        }
                    }
                }
                // if the current client is the host, show start game button
                if SERVER_NAME() == local_login_name_session() && all_players_have_character_name
                    && (server_data_snap.core_game_data.game_phase == GamePhase::InitGame
                        || server_data_snap.core_game_data.game_phase == GamePhase::Loading
                            && server_data_snap.core_game_data.players_nb
                                == server_data_snap.players_data.players_info.len() as i64)
                {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| async move {
                            send_start_game(socket).await;
                        },
                        "▶ Start Game"
                    }
                }
                CharacterSelect {}
            }
        } else if server_data_snap.core_game_data.game_phase == GamePhase::Running {
            // check if there is more characters in game than users
            if server_data_snap.core_game_data.game_manager.pm.active_heroes.len()
                <= server_data_snap.players_data.players_info.len()
            {
                RunningGamePage {}
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
        } else if server_data_snap.core_game_data.game_phase == GamePhase::Ended {
            ButtonLink {
                target: Route::Home {}.into(),
                name: "No more game, back to home".to_string(),
            }
        } else if server_data_snap.core_game_data.game_phase == GamePhase::Default {

        }
    }
}

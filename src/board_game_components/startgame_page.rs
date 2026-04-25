use crate::board_game_components::game_sheets::GameSheets;
use crate::common::{Route, SERVER_NAME};
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::websocket_handler::msg_from_client::send_disconnect_from_server_data;
use crate::{
    board_game_components::gameboard::GameBoard,
    components::{
        button::{Button, ButtonVariant},
        separator::Separator,
    },
};
use dioxus::fullstack::{CborEncoding, UseWebsocket};
use dioxus::prelude::*;
use lib_rpg::server::game_state::GameStatus;
use lib_rpg::server::server_manager::ServerData;

#[component]
pub fn QuitGameButton() -> Element {
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    rsx! {
        Button {
            variant: ButtonVariant::Primary,
            onclick: move |_| {
                async move {
                    send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                    let navigator = use_navigator();
                    navigator.push(Route::Home {});
                }
            },
            "Quit"
        }
    }
}

/// New game
#[component]
pub fn RunningGamePage() -> Element {
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let snap_server_data = server_data();

    rsx! {
        if server_data().core_game_data.game_manager.game_state.status == GameStatus::EndOfGame {
            h1 { "Game Over" }
            h2 { "Remaining players: {server_data().players_data.players_info.len()}" }

            QuitGameButton {}

            if server_data().players_data.owner_player_name == local_login_name_session() {
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| async move {
                        let _ = socket.send(ClientEvent::ReplayGame(SERVER_NAME())).await;
                    },
                    "Replay game"
                }
            }
        }
        if server_data().core_game_data.game_manager.game_state.status
            == GameStatus::EndOfScenario
        {

            div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",

                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| async move {
                        let _ = socket.send(ClientEvent::LoadNextScenario(SERVER_NAME())).await;
                    },
                    "Load next scenario"
                }
                Separator {
                    style: "margin: 10px 0;",
                    horizontal: false,
                    decorative: true,
                }
                QuitGameButton {}
            }
            h3 { "Loots:" }
            // display loots by character and their class
            for l in snap_server_data.core_game_data.game_manager.current_scenario.loots.iter() {
                div { style: "display: flex; flex-direction: row; gap: 10px;",
                    h4 { "{l.format_loot()}" }
                }
            }
            // display level upgrades
            h3 { "Level upgrades :" }
            div { style: "display: flex; flex-direction: row; gap: 10px;",
                h4 { dangerous_inner_html: "{snap_server_data.core_game_data.game_manager.end_of_scenario.to_formatted_string(true)}" }
            }
        } else {
            Separator {
                style: "margin: 10px 0;",
                horizontal: true,
                decorative: true,
            }
            div {
                div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",
                    GameSheets {}
                    {}
                    h4 { "Turn: {server_data().core_game_data.game_manager.game_state.current_turn_nb}" }
                }
                Separator {
                    style: "margin: 10px 0;",
                    horizontal: true,
                    decorative: true,
                }
                GameBoard {}
            }
        }

    }
}

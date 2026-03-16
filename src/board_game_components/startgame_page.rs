use crate::board_game_components::game_sheets::GameSheets;
use crate::common::{Route, SERVER_NAME};
use crate::components::label::Label;
use crate::components::scroll_area::ScrollArea;
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::websocket_handler::msg_from_client::{
    request_save_game, send_disconnect_from_server_data,
};
use crate::widgets::tab::TabDemo;
use crate::{
    board_game_components::gameboard::GameBoard,
    components::{
        button::{Button, ButtonVariant},
        separator::Separator,
        sheet::*,
    },
};
use dioxus::fullstack::{CborEncoding, UseWebsocket};
use dioxus::prelude::*;
use dioxus_primitives::scroll_area::ScrollDirection;
use lib_rpg::character_mod::character::Character;
use lib_rpg::server::game_state::GameStatus;
use lib_rpg::server::server_manager::ServerData;

/// New game
#[component]
pub fn RunningGamePage() -> Element {
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    rsx! {
        if server_data().core_game_data.game_manager.game_state.status == GameStatus::EndOfGame {
            h1 { "Game Over" }
            h2 { "Remaining players: {server_data().players_data.players_info.len()}" }

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
        Separator { style: "margin: 10px 0;", horizontal: true, decorative: true }
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

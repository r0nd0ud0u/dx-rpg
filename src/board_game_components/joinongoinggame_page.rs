use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};

use crate::{
    application::{self, Application},
    board_game_components::{
        common_comp::ButtonLink,
        msg_from_client::{send_join_server_data, send_start_game},
    },
    common::Route,
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        game_state::OnGoingGame,
    },
};
use dioxus::logger::tracing;

#[component]
pub fn JoinOngoingGame() -> Element {
    // contexts
    let ongoing_games_sig = use_context::<Signal<Vec<OnGoingGame>>>();
    let local_login_name_session = use_context::<Signal<String>>();

    // snapshots
    let snap_ongoing_games = ongoing_games_sig().clone();

    rsx! {
        div { class: "ongoing-games-container",
            h4 { "Ongoing Games" }
            for game in snap_ongoing_games.iter() {
                GamePanel {
                    server_name: game.server_name.clone(),
                    player_name: local_login_name_session().clone(),
                
                }
            }
        }
    }
}

#[component]
pub fn GamePanel(server_name: String, player_name: String) -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    rsx! {
        div { class: "ongoing-game-item",
            ButtonLink {
                target: Route::LobbyPage {}.into(),
                name: server_name.clone(),
                onclick: move |_| {
                    let l_server_name = server_name.clone();
                    let l_player_name = player_name.clone();
                    async move {
                        send_join_server_data(socket, &l_server_name, &l_player_name).await;
                    }
                },
            }
        }
    }
}

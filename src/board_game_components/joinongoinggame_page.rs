use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use lib_rpg::server::server_manager::OnGoingGame;

use crate::{
    board_game_components::common_comp::ButtonLink,
    common::Route,
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::send_join_server_data,
    },
};

#[component]
pub fn JoinOngoingGame() -> Element {
    // contexts
    let ongoing_games_sig = use_context::<Signal<Vec<OnGoingGame>>>();
    let local_login_name_session = use_context::<Signal<String>>();

    // snapshots
    let snap_ongoing_games = ongoing_games_sig().clone();

    rsx! {
        div { class: "ongoing-games-container",
            h2 { class: "rpg-title", "🗺️ Ongoing Adventures" }
            if snap_ongoing_games.is_empty() {
                p { class: "rpg-subtitle", "No games running yet. Create one!" }
            } else {
                div { class: "games-grid",
                    for game in snap_ongoing_games.iter() {
                        GamePanel {
                            server_name: game.server_name.clone(),
                            player_name: local_login_name_session().clone(),
                        }
                    }
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

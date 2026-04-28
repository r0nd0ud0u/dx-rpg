use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus::{fullstack::UseWebsocket, logger::tracing};

use crate::utils::server_file_utils;
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::{
    common::Route,
    components::button::{Button, ButtonVariant},
};

#[component]
pub fn LoadGame() -> Element {
    // contexts
    let games_list = use_context::<Signal<Vec<std::path::PathBuf>>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    // states
    let mut active_button: Signal<i64> = use_signal(|| -1);
    let navigator = use_navigator();

    // snap
    let games_list_snap = games_list();

    // create a list by reading the games_list signal and  splitting the path by "/" or "\" and taking the last element (the game name)
    let games_list_split = games_list_snap
        .iter()
        .map(|path| {
            // split trying both separators to be sure
            let path_str = path.to_string_lossy();
            let split_by_slash: Vec<&str> = path_str.split('/').collect();
            let split_by_backslash: Vec<&str> = path_str.split('\\').collect();
            let game_name = if split_by_slash.len() > split_by_backslash.len() {
                split_by_slash.last().unwrap_or(&"")
            } else {
                split_by_backslash.last().unwrap_or(&"")
            };
            game_name.to_string()
        })
        .collect::<Vec<String>>();

    let save_count = games_list_split.len();
    let plural = if save_count != 1 { "s" } else { "" };

    rsx! {
        div { class: "home-container",
            h2 { class: "rpg-title", "💾 Load Game" }
            p { class: "rpg-subtitle", "{save_count} saved adventure{plural}" }

            div { class: "load-game-card",
                if games_list_split.is_empty() {
                    div { class: "load-empty",
                        span { "📂" }
                        p { "No saved games found" }
                    }
                } else {
                    div { class: "game-list",
                        for (index, game_name) in games_list_split.iter().enumerate() {
                            button {
                                class: if active_button() as usize == index { "game-item selected" } else { "game-item" },
                                onclick: move |_| async move { active_button.set(index as i64) },
                                span { class: "game-item-icon", "🎮" }
                                span { class: "game-item-name", "{game_name}" }
                                if active_button() as usize == index {
                                    span { class: "game-item-check", "✓" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "load-actions",
                Button {
                    variant: ButtonVariant::GreenType,
                    disabled: active_button() == -1,
                    onclick: move |_| {
                        let cur_game = games_list().get(active_button() as usize).unwrap().to_owned();
                        async move {
                            let _ = socket
                                .clone()
                                .send(
                                    ClientEvent::LoadGame(cur_game.clone(), local_login_name_session()),
                                )
                                .await;
                            navigator.push(Route::LobbyPage {});
                        }
                    },
                    "▶ Load Game"
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    disabled: active_button() == -1,
                    onclick: move |_| {
                        let cur_game = games_list().get(active_button() as usize).unwrap().to_owned();
                        async move {
                            match server_file_utils::delete_game(cur_game.clone()).await {
                                Ok(_) => {
                                    let _ = socket
                                        .clone()
                                        .send(
                                            ClientEvent::RequestSavedGameList(
                                                local_login_name_session().clone(),
                                            ),
                                        )
                                        .await;
                                }
                                Err(e) => {
                                    tracing::error!("Error deleting game: {}", e);
                                    return;
                                }
                            };
                            active_button.set(-1);
                        }
                    },
                    "🗑 Delete"
                }
            }
        }
    }
}

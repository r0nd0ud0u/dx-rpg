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
    let games_list = use_context::<Signal<Vec<std::path::PathBuf>>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let mut selected: Signal<Option<usize>> = use_signal(|| None);
    let navigator = use_navigator();

    let games_list_snap = games_list();
    let save_count = games_list_snap.len();
    let plural = if save_count != 1 { "s" } else { "" };

    /// Extract the game name from a PathBuf (last path component).
    fn extract_name(path: &std::path::Path) -> String {
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned())
    }

    rsx! {
        div { class: "home-container",
            h2 { class: "rpg-title", "💾 Load Game" }
            p { class: "rpg-subtitle", "{save_count} saved adventure{plural}" }

            if games_list_snap.is_empty() {
                div { class: "load-empty",
                    span { "📂" }
                    p { "No saved games found." }
                    p { style: "font-size:.82rem; color:var(--rpg-text-muted);",
                        "Create a new game first."
                    }
                }
            } else {
                div { class: "save-slot-grid",
                    for (index, path) in games_list_snap.iter().enumerate() {
                        {
                            let game_name = extract_name(path);
                            let is_selected = selected() == Some(index);
                            rsx! {
                                div {
                                    class: if is_selected { "save-slot-card save-slot-occupied selected" } else { "save-slot-card save-slot-occupied" },
                                    onclick: move |_| {
                                        if selected() == Some(index) {
                                            selected.set(None);
                                        } else {
                                            selected.set(Some(index));
                                        }
                                    },
                                    span { class: "save-slot-icon", "🎮" }
                                    div { class: "save-slot-info",
                                        span { class: "save-slot-name", "{game_name}" }
                                    }
                                    if is_selected {
                                        div { class: "save-slot-actions",
                                            Button {
                                                variant: ButtonVariant::GreenType,
                                                onclick: move |_| {
                                                    let cur_game = games_list()
                                                        .get(index)
                                                        .unwrap()
                                                        .to_owned();
                                                    let player = local_login_name_session();
                                                    async move {
                                                        let _ = socket
                                                            .send(ClientEvent::LoadGame(
                                                                cur_game,
                                                                player,
                                                            ))
                                                            .await;
                                                        navigator.push(Route::LobbyPage {});
                                                    }
                                                },
                                                "▶ Load"
                                            }
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                onclick: move |_| {
                                                    let cur_game = games_list()
                                                        .get(index)
                                                        .unwrap()
                                                        .to_owned();
                                                    async move {
                                                        match server_file_utils::delete_game(cur_game).await {
                                                            Ok(_) => {
                                                                let _ = socket
                                                                    .send(
                                                                        ClientEvent::RequestSavedGameList(
                                                                            local_login_name_session()
                                                                                .clone(),
                                                                        ),
                                                                    )
                                                                    .await;
                                                            }
                                                            Err(e) => {
                                                                tracing::error!("Error deleting game: {}", e);
                                                            }
                                                        }
                                                        selected.set(None);
                                                    }
                                                },
                                                "🗑 Delete"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus::{fullstack::UseWebsocket, logger::tracing};

use crate::utils::server_file_utils::{SaveSlotInfo, delete_game, get_save_slots};
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::{
    common::Route,
    components::button::{Button, ButtonVariant},
};

#[component]
pub fn LoadGame() -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let mut slots: Signal<Vec<SaveSlotInfo>> = use_signal(Vec::new);
    let mut selected: Signal<Option<usize>> = use_signal(|| None);
    let mut error_msg: Signal<String> = use_signal(String::new);
    let navigator = use_navigator();

    let player = local_login_name_session();
    use_effect(move || {
        let player_name = player.clone();
        spawn(async move {
            match get_save_slots(player_name).await {
                Ok(s) => slots.set(s.into_iter().filter(|s| !s.name.is_empty()).collect()),
                Err(e) => error_msg.set(format!("Failed to load saves: {e}")),
            }
        });
    });

    let occupied_slots = slots();
    let save_count = occupied_slots.len();
    let plural = if save_count != 1 { "s" } else { "" };

    rsx! {
        div { class: "home-container",
            h2 { class: "rpg-title", "💾 Load Game" }
            p { class: "rpg-subtitle", "{save_count} saved adventure{plural}" }

            if !error_msg().is_empty() {
                p { class: "admin-answer-error", "{error_msg}" }
            }

            if occupied_slots.is_empty() {
                div { class: "load-empty",
                    span { "📂" }
                    p { "No saved games found." }
                    p { style: "font-size:.82rem; color:var(--rpg-text-muted);",
                        "Create a new game first."
                    }
                }
            } else {
                div { class: "save-slot-grid",
                    for (index, slot) in occupied_slots.iter().enumerate() {
                        {
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
                                    span { class: "save-slot-icon", "💾" }
                                    div { class: "save-slot-info",
                                        span { class: "save-slot-name", "{slot.name}" }
                                        if !slot.current_scenario.is_empty() {
                                            span { class: "save-slot-scenario",
                                                "📜 {slot.current_scenario} (Lvl {slot.scenario_level})"
                                            }
                                        }
                                        span { class: "save-slot-date", "🕐 {slot.last_saved}" }
                                        div { class: "save-slot-meta",
                                            if slot.is_single_player {
                                                span { class: "save-slot-mode", "🎮 Solo" }
                                            } else {
                                                span { class: "save-slot-mode", "👥 Multi ({slot.players_nb}p)" }
                                            }
                                            if !slot.universe.is_empty() {
                                                span { class: "save-slot-universe", "🌐 {slot.universe}" }
                                            }
                                        }
                                    }
                                    if is_selected {
                                        div { class: "save-slot-actions",
                                            Button {
                                                variant: ButtonVariant::GreenType,
                                                onclick: {
                                                    let path = slot.path.clone();
                                                    let player = local_login_name_session();
                                                    move |_| {
                                                        let p = path.clone();
                                                        let pl = player.clone();
                                                        async move {
                                                            let _ = socket
                                                                .send(ClientEvent::LoadGame(p, pl))
                                                                .await;
                                                            navigator.push(Route::LobbyPage {});
                                                        }
                                                    }
                                                },
                                                "▶ Load"
                                            }
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                onclick: {
                                                    let path = slot.path.clone();
                                                    move |_| {
                                                        let p = path.clone();
                                                        async move {
                                                            match delete_game(p).await {
                                                                Ok(_) => {
                                                                    match get_save_slots(local_login_name_session().clone()).await {
                                                                        Ok(s) => {
                                                                            slots
                                                                                .set(s.into_iter().filter(|s| !s.name.is_empty()).collect())
                                                                        }
                                                                        Err(e) => tracing::error!("Failed to reload slots: {}", e),
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    tracing::error!("Error deleting game: {}", e);
                                                                }
                                                            }
                                                            selected.set(None);
                                                        }
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

use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus::{fullstack::UseWebsocket, logger::tracing};
use dioxus_i18n::t;

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
                Err(e) => error_msg.set(t!("loadgame-fetch-error", error: e.to_string())),
            }
        });
    });

    let occupied_slots = slots();
    let save_count = occupied_slots.len();
    let count_label = if save_count == 1 {
        t!("loadgame-count-one", count: save_count as i64)
    } else {
        t!("loadgame-count-other", count: save_count as i64)
    };

    rsx! {
        div { class: "home-container",
            h2 { class: "rpg-title", {t!("loadgame-title")} }
            p { class: "rpg-subtitle", "{count_label}" }

            if !error_msg().is_empty() {
                p { class: "admin-answer-error", "{error_msg}" }
            }

            if occupied_slots.is_empty() {
                div { class: "load-empty",
                    span { "📂" }
                    p { {t!("loadgame-empty")} }
                    p { style: "font-size:.82rem; color:var(--rpg-text-muted);",
                        {t!("loadgame-empty-hint")}
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
                                                {
                                                    t!(
                                                        "loadgame-slot-scenario", scenario : slot.current_scenario
                                                        .clone(), level : slot.scenario_level as i64
                                                    )
                                                }
                                            }
                                        }
                                        span { class: "save-slot-date", "🕐 {slot.last_saved}" }
                                        div { class: "save-slot-meta",
                                            if slot.is_single_player {
                                                span { class: "save-slot-mode", {t!("loadgame-mode-solo")} }
                                            } else {
                                                span { class: "save-slot-mode",
                                                    {t!("loadgame-mode-multi", players : slot.players_nb)}
                                                }
                                            }
                                            if !slot.universe.is_empty() {
                                                span { class: "save-slot-universe",
                                                    {t!("loadgame-universe", universe : slot.universe.clone())}
                                                }
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
                                                {t!("loadgame-load-button")}
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
                                                {t!("loadgame-delete-button")}
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

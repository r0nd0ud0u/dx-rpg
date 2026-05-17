use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};

use crate::{
    auth_manager::server_fn::get_available_universes,
    common::Route,
    components::button::{Button, ButtonVariant},
    utils::server_file_utils::{SaveSlotInfo, delete_game, get_save_slots},
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::{request_update_saved_game_list_display, send_initialize_game},
    },
};

/// New game: the player picks a save slot (empty or occupied) to create / overwrite a game.
#[component]
pub fn CreateServer() -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let navigator = use_navigator();

    // Save-slot state
    let mut slots: Signal<Vec<SaveSlotInfo>> = use_signal(Vec::new);
    let mut selected_slot: Signal<Option<usize>> = use_signal(|| None);
    let mut confirm_overwrite: Signal<bool> = use_signal(|| false);
    let mut error_msg: Signal<String> = use_signal(String::new);

    // Universe selection state
    let mut universes: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_universe: Signal<String> = use_signal(String::new);

    // Load slots and universes once on mount
    let player = local_login_name_session();
    use_effect(move || {
        let player_name = player.clone();
        spawn(async move {
            match get_save_slots(player_name).await {
                Ok(s) => slots.set(s),
                Err(e) => error_msg.set(format!("Failed to load saves: {e}")),
            }
            match get_available_universes().await {
                Ok(u) => {
                    // Auto-select the first universe if only one exists
                    if u.len() == 1 {
                        selected_universe.set(u[0].clone());
                    }
                    universes.set(u);
                }
                Err(e) => tracing::error!("get_available_universes: {e}"),
            }
        });
    });

    let start_game_in_slot = move |idx: usize| {
        let slot = slots().get(idx).cloned();
        let user_name = local_login_name_session();
        let universe = selected_universe();
        async move {
            // If the slot is occupied, delete the old data first
            if let Some(s) = slot {
                if !s.path.as_os_str().is_empty() {
                    let _ = delete_game(s.path).await;
                }
            }
            send_initialize_game(&user_name, &universe, socket).await;
            navigator.push(Route::LobbyPage {});
        }
    };

    rsx! {
        div { class: "home-container",
            h1 { class: "rpg-title", "🏰 Choose a Save Slot" }
            p { class: "rpg-subtitle", "Select an empty slot or overwrite an existing save" }

            if !error_msg().is_empty() {
                p { class: "admin-answer-error", "{error_msg}" }
            }

            // Universe selector — only shown when there are multiple universes
            if universes().len() > 1 {
                div { class: "universe-selector",
                    p { class: "rpg-subtitle", "🌍 Choose a Universe" }
                    div { class: "universe-grid",
                        for uni in universes() {
                            div {
                                class: if selected_universe() == uni {
                                    "universe-card universe-card-selected"
                                } else {
                                    "universe-card"
                                },
                                onclick: {
                                    let uni = uni.clone();
                                    move |_| selected_universe.set(uni.clone())
                                },
                                span { class: "universe-icon", "🌐" }
                                span { class: "universe-name", "{uni}" }
                            }
                        }
                    }
                }
            }

            div { class: "save-slot-grid",
                for (idx , slot) in slots().iter().enumerate() {
                    if slot.name.is_empty() {
                        // Empty slot
                        div {
                            class: "save-slot-card save-slot-empty",
                            onclick: move |_| async move { start_game_in_slot(idx).await },
                            span { class: "save-slot-icon", "➕" }
                            span { class: "save-slot-label", "Empty Slot {idx + 1}" }
                        }
                    } else {
                        // Occupied slot
                        div {
                            class: if selected_slot() == Some(idx) {
                                "save-slot-card save-slot-occupied selected"
                            } else {
                                "save-slot-card save-slot-occupied"
                            },
                            onclick: move |_| {
                                if selected_slot() == Some(idx) {
                                    selected_slot.set(None);
                                    confirm_overwrite.set(false);
                                } else {
                                    selected_slot.set(Some(idx));
                                    confirm_overwrite.set(false);
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
                            }
                            if selected_slot() == Some(idx) {
                                div { class: "save-slot-actions",
                                    Button {
                                        variant: ButtonVariant::GreenType,
                                        onclick: {
                                            let path = slot.path.clone();
                                            let user = local_login_name_session();
                                            move |_| {
                                                let _ = socket.clone();
                                                let p = path.clone();
                                                let u = user.clone();
                                                let uni = selected_universe();
                                                async move {
                                                    if !p.as_os_str().is_empty() {
                                                        let _ = delete_game(p).await;
                                                    }
                                                    send_initialize_game(&u, &uni, socket).await;
                                                    navigator.push(Route::LobbyPage {});
                                                }
                                            }
                                        },
                                        "▶ Overwrite & Play"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "action-grid",
                div { class: "action-card",
                    span { class: "action-icon", "💾" }
                    Link {
                        class: "header-text",
                        to: Route::LoadGame {},
                        onclick: move |_| {
                            let user_name = local_login_name_session();
                            async move {
                                request_update_saved_game_list_display(socket, &user_name).await;
                            }
                        },
                        "Load Game"
                    }
                    p { class: "action-desc", "Continue a saved adventure" }
                }
            }
        }
    }
}

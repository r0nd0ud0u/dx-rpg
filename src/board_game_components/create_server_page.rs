use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use lib_rpg::{character_mod::character::Character, common::constants::stats_const::HP};

use crate::{
    auth_manager::server_fn::get_available_universes,
    common::{PATH_IMG, Route},
    components::button::{Button, ButtonVariant},
    utils::server_file_utils::{SaveSlotInfo, delete_game, get_save_slots},
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::{request_update_saved_game_list_display, send_initialize_game},
    },
};

#[component]
pub fn CreateServer() -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let all_characters = use_context::<Signal<Vec<Character>>>();
    let navigator = use_navigator();

    let mut slots: Signal<Vec<SaveSlotInfo>> = use_signal(Vec::new);
    let mut selected_slot: Signal<Option<usize>> = use_signal(|| None);
    let mut error_msg: Signal<String> = use_signal(String::new);

    let mut universes: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_universe: Signal<String> = use_signal(String::new);
    let mut is_single_player: Signal<bool> = use_signal(|| false);

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
                    if !u.is_empty() {
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
        let single = is_single_player();
        async move {
            if let Some(s) = slot {
                if !s.path.as_os_str().is_empty() {
                    let _ = delete_game(s.path).await;
                }
            }
            send_initialize_game(&user_name, &universe, single, socket).await;
            navigator.push(Route::LobbyPage {});
        }
    };

    // Hero characters for preview
    let hero_chars: Vec<Character> = all_characters()
        .into_iter()
        .filter(|c| c.kind == lib_rpg::character_mod::character::CharacterKind::Hero)
        .collect();

    rsx! {
        div { class: "home-container",
            h1 { class: "rpg-title", "🏰 Create a Game" }

            if !error_msg().is_empty() {
                p { class: "admin-answer-error", "{error_msg}" }
            }

            // ── Step 1: Game Mode ────────────────────────────────────────────
            div { class: "create-server-section",
                p { class: "create-server-section-title", "1️⃣ Game Mode" }
                div { class: "mode-toggle",
                    button {
                        class: if !is_single_player() { "mode-btn mode-btn-active" } else { "mode-btn" },
                        onclick: move |_| is_single_player.set(false),
                        "👥 Multiplayer"
                    }
                    button {
                        class: if is_single_player() { "mode-btn mode-btn-active" } else { "mode-btn" },
                        onclick: move |_| is_single_player.set(true),
                        "🎮 Single Player"
                    }
                }
                p { class: "create-server-mode-hint",
                    if is_single_player() {
                        "One player controls all heroes."
                    } else {
                        "Each connected player picks one hero."
                    }
                }
            }

            // ── Step 2: Universe ─────────────────────────────────────────────
            if universes().len() > 1 {
                div { class: "create-server-section",
                    p { class: "create-server-section-title", "2️⃣ Choose a Universe" }
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

            // ── Step 3: Character Preview ────────────────────────────────────
            div { class: "create-server-section",
                p { class: "create-server-section-title",
                    if is_single_player() { "3️⃣ Characters Available" } else { "3️⃣ Choose Your Character (in Lobby)" }
                }
                if is_single_player() {
                    p { class: "create-server-mode-hint",
                        "You will pick your heroes in the Lobby after creating the game."
                    }
                }
                div { class: "char-preview-grid",
                    for c in hero_chars {
                        div { class: "char-preview-card",
                            img {
                                class: "char-preview-portrait",
                                src: format!("{}/{}.png", PATH_IMG, c.photo_name),
                                alt: "{c.db_full_name}",
                            }
                            div { class: "char-preview-info",
                                span { class: "char-preview-name", "{c.db_full_name}" }
                                span { class: "char-preview-class",
                                    "{c.class.to_emoji()} {c.class.to_str()} · Lv {c.level}"
                                }
                                {
                                    let max_hp = c.stats.all_stats.get(HP).map(|a| a.max).unwrap_or(0);
                                    rsx! {
                                        span { class: "char-preview-hp", "❤️ {max_hp}" }
                                    }
                                }
                                if !c.description.is_empty() {
                                    p { class: "char-preview-desc", "{c.description}" }
                                }
                            }
                        }
                    }
                }
            }

            // ── Step 4: Save Slot ────────────────────────────────────────────
            div { class: "create-server-section",
                p { class: "create-server-section-title", "4️⃣ Choose a Save Slot" }
                div { class: "save-slot-grid",
                    for (idx, slot) in slots().iter().enumerate() {
                        if slot.name.is_empty() {
                            div {
                                class: "save-slot-card save-slot-empty",
                                onclick: move |_| async move { start_game_in_slot(idx).await },
                                span { class: "save-slot-icon", "➕" }
                                span { class: "save-slot-label", "Empty Slot {idx + 1}" }
                            }
                        } else {
                            div {
                                class: if selected_slot() == Some(idx) {
                                    "save-slot-card save-slot-occupied selected"
                                } else {
                                    "save-slot-card save-slot-occupied"
                                },
                                onclick: move |_| {
                                    if selected_slot() == Some(idx) {
                                        selected_slot.set(None);
                                    } else {
                                        selected_slot.set(Some(idx));
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
                                                    let p = path.clone();
                                                    let u = user.clone();
                                                    let uni = selected_universe();
                                                    let single = is_single_player();
                                                    async move {
                                                        if !p.as_os_str().is_empty() {
                                                            let _ = delete_game(p).await;
                                                        }
                                                        send_initialize_game(&u, &uni, single, socket).await;
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

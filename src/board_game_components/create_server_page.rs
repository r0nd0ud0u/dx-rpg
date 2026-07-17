use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_i18n::t;

use crate::{
    common::Route,
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
    let navigator = use_navigator();

    let mut slots: Signal<Vec<SaveSlotInfo>> = use_signal(Vec::new);
    let mut selected_slot: Signal<Option<usize>> = use_signal(|| None);
    let mut error_msg: Signal<String> = use_signal(String::new);

    let mut is_single_player: Signal<bool> = use_signal(|| false);

    // Reading local_login_name_session() *inside* the effect (not captured outside)
    // subscribes the effect to the signal so it re-runs whenever the login name
    // changes — including when it is restored from localStorage after SSR hydration.
    use_effect(move || {
        let player_name = local_login_name_session();
        spawn(async move {
            match get_save_slots(player_name).await {
                Ok(s) => slots.set(s),
                Err(e) => error_msg.set(t!("loadgame-fetch-error", error: e.to_string())),
            }
        });
    });

    let start_game_in_slot = move |idx: usize| {
        let slot = slots().get(idx).cloned();
        let user_name = local_login_name_session();
        let single = is_single_player();
        async move {
            if let Some(s) = slot
                && !s.path.as_os_str().is_empty()
            {
                let _ = delete_game(s.path).await;
            }
            // Universe will be chosen in the lobby; pass empty string to load all universes
            send_initialize_game(&user_name, "", single, socket).await;
            navigator.push(Route::LobbyPage {});
        }
    };

    rsx! {
        div { class: "home-container",
            h1 { class: "rpg-title", {t!("create-server-title")} }

            if !error_msg().is_empty() {
                p { class: "admin-answer-error", "{error_msg}" }
            }

            // ── Step 1: Game Mode ────────────────────────────────────────────
            div { class: "create-server-section",
                p { class: "create-server-section-title", {t!("create-server-step1")} }
                div { class: "mode-toggle",
                    button {
                        class: if !is_single_player() { "mode-btn mode-btn-active" } else { "mode-btn" },
                        onclick: move |_| is_single_player.set(false),
                        {t!("create-server-multiplayer")}
                    }
                    button {
                        class: if is_single_player() { "mode-btn mode-btn-active" } else { "mode-btn" },
                        onclick: move |_| is_single_player.set(true),
                        {t!("create-server-singleplayer")}
                    }
                }
                p { class: "create-server-mode-hint",
                    if is_single_player() {
                        {t!("create-server-singleplayer-hint")}
                    } else {
                        {t!("create-server-multiplayer-hint")}
                    }
                }
            }

            // ── Step 2: Save Slot ────────────────────────────────────────────
            div { class: "create-server-section",
                p { class: "create-server-section-title", {t!("create-server-step2")} }
                div { class: "save-slot-grid",
                    for (idx, slot) in slots().iter().enumerate() {
                        if slot.name.is_empty() {
                            div {
                                class: "save-slot-card save-slot-empty",
                                onclick: move |_| async move { start_game_in_slot(idx).await },
                                span { class: "save-slot-icon", "➕" }
                                span { class: "save-slot-label",
                                    {t!("create-server-empty-slot", index : (idx + 1) as i64)}
                                }
                            }
                        } else {
                            div {
                                class: if selected_slot() == Some(idx) { "save-slot-card save-slot-occupied selected" } else { "save-slot-card save-slot-occupied" },
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
                                                    let single = is_single_player();
                                                    async move {
                                                        if !p.as_os_str().is_empty() {
                                                            let _ = delete_game(p).await;
                                                        }
                                                        send_initialize_game(&u, "", single, socket).await;
                                                        navigator.push(Route::LobbyPage {});
                                                    }
                                                }
                                            },
                                            {t!("create-server-overwrite-play")}
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
                        {t!("create-server-load-game")}
                    }
                    p { class: "action-desc", {t!("create-server-load-game-desc")} }
                }
            }
        }
    }
}

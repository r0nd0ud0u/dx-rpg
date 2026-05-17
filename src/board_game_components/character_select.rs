use std::collections::HashMap;

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use lib_rpg::{
    character_mod::character::Character,
    common::constants::stats_const::HP,
    server::server_manager::{GamePhase, ServerData},
};

use crate::{
    common::PATH_IMG,
    websocket_handler::event::{ClientEvent, ServerEvent},
};

#[component]
pub fn CharacterSelect() -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let local_name = local_login_name_session();
    let server_data_snap = server_data();

    if server_data_snap.players_data.players_info.is_empty() {
        return rsx! {};
    }

    let connected: HashMap<String, String> = server_data_snap
        .core_game_data
        .heroes_chosen
        .iter()
        .map(|(key, value)| {
            let status = if server_data_snap.players_data.players_info.contains_key(key) {
                "✅"
            } else {
                "❌"
            };
            (key.clone(), format!("{} {}", value, status))
        })
        .collect();

    let players_except_current_client: HashMap<String, String> = server_data_snap
        .players_data
        .players_info
        .iter()
        .filter(|(k, _)| k.as_str() != local_name.as_str())
        .map(|(k, v)| {
            let name = v
                .character_id_names
                .first()
                .unwrap_or(&"No character selected".to_string())
                .split("_#")
                .next()
                .unwrap_or("No character selected")
                .to_string();
            (k.clone(), name)
        })
        .collect();

    rsx! {
        div { class: "char-select-container",
            h3 { class: "char-select-title", "Players" }

            // Current player — character picker cards
            if server_data_snap.core_game_data.game_phase == GamePhase::InitGame {
                CharacterCardGrid { player_name: local_login_name_session().clone() }
            } else {
                div { class: "char-select-chosen-list",
                    for (player , choice) in connected.clone() {
                        div { class: "char-select-chosen-row",
                            span { class: "char-select-player-name", "{player}" }
                            span { class: "char-select-chosen-char", "{choice}" }
                        }
                    }
                }
            }

            // Other players already in the lobby
            if server_data_snap.core_game_data.game_phase == GamePhase::InitGame
                && !players_except_current_client.is_empty()
            {
                div { class: "char-select-others",
                    p { class: "char-select-others-title", "Other players:" }
                    for (player , choice) in players_except_current_client.clone() {
                        div { class: "char-select-chosen-row",
                            span { class: "char-select-player-name", "{player}" }
                            span {
                                class: if choice == "No character selected" {
                                    "char-select-waiting"
                                } else {
                                    "char-select-chosen-char"
                                },
                                "{choice}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CharacterCardGrid(player_name: String) -> Element {
    let server_data = use_context::<Signal<ServerData>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let all_characters = use_context::<Signal<Vec<Character>>>();

    // Get current selection for this player
    let current_choice = server_data()
        .core_game_data
        .heroes_chosen
        .get(&player_name)
        .cloned()
        .unwrap_or_default();

    let hero_chars: Vec<Character> = all_characters()
        .into_iter()
        .filter(|c| c.kind == lib_rpg::character_mod::character::CharacterKind::Hero)
        .collect();

    rsx! {
        div { class: "char-card-grid",
            for c in hero_chars {
                {
                    let is_selected = current_choice == c.db_full_name;
                    let cname = c.db_full_name.clone();
                    let server_name = server_data().core_game_data.server_name.clone();
                    let pname = player_name.clone();
                    let max_hp = c.stats.all_stats.get(HP).map(|s| s.max).unwrap_or(0);
                    let desc = c.description.clone();
                    rsx! {
                        div {
                            class: if is_selected { "char-card char-card-selected" } else { "char-card" },
                            onclick: move |_| {
                                let cn = cname.clone();
                                let sn = server_name.clone();
                                let pn = pname.clone();
                                async move {
                                    tracing::info!("Selected character: {}", cn);
                                    let _ = socket
                                        .send(ClientEvent::AddCharacterOnServerData(sn, pn, cn))
                                        .await;
                                }
                            },
                            // Portrait
                            div { class: "char-card-portrait",
                                img {
                                    src: format!("{}/{}.png", PATH_IMG, c.photo_name),
                                    class: "char-card-img",
                                    alt: "{c.db_full_name}",
                                }
                            }
                            // Info
                            div { class: "char-card-info",
                                span { class: "char-card-name", "{c.db_full_name}" }
                                div { class: "char-card-badges",
                                    span { class: "char-card-class", "{c.class.to_emoji()} {c.class.to_str()}" }
                                    span { class: "char-card-level", "Lv {c.level}" }
                                }
                                div { class: "char-card-hp",
                                    span { class: "char-card-hp-label", "HP" }
                                    span { class: "char-card-hp-val", "{max_hp}" }
                                }
                                if !desc.is_empty() {
                                    p { class: "char-card-desc", "{desc}" }
                                }
                            }
                            if is_selected {
                                div { class: "char-card-check", "✓" }
                            }
                        }
                    }
                }
            }
        }
    }
}

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

fn strip_id_suffix(id_name: &str) -> &str {
    id_name.split("_#").next().unwrap_or(id_name)
}

#[component]
pub fn CharacterSelect(universe: String) -> Element {
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let local_name = local_login_name_session();
    let server_data_snap = server_data();

    if server_data_snap.players_data.players_info.is_empty() {
        return rsx! {};
    }

    let chosen: HashMap<String, String> = server_data_snap
        .core_game_data
        .heroes_chosen
        .iter()
        .map(|(k, v)| (k.clone(), strip_id_suffix(v).to_string()))
        .collect();

    let mut others: Vec<(String, String)> = server_data_snap
        .players_data
        .players_info
        .iter()
        .filter(|(k, _)| k.as_str() != local_name.as_str())
        .map(|(k, _)| {
            let char_name = chosen.get(k).cloned().unwrap_or_else(|| "—".to_string());
            (k.clone(), char_name)
        })
        .collect();
    others.sort_by(|a, b| a.0.cmp(&b.0));

    let is_single = server_data_snap.core_game_data.is_single_player;

    rsx! {
        div { class: "char-select-container",
            h3 { class: "char-select-title",
                if is_single {
                    "🎮 Single Player — Choose Your Heroes"
                } else {
                    "👥 Choose Your Character"
                }
            }

            if server_data_snap.core_game_data.game_phase == GamePhase::InitGame {
                CharacterCardGrid {
                    player_name: local_name.clone(),
                    is_single_player: is_single,
                    universe: universe.clone(),
                }
            } else {
                div { class: "char-select-chosen-list",
                    for (player, choice) in chosen.clone() {
                        div { class: "char-select-chosen-row",
                            span { class: "char-select-player-name", "{player}" }
                            span { class: "char-select-chosen-char", "{choice}" }
                        }
                    }
                }
            }

            if server_data_snap.core_game_data.game_phase == GamePhase::InitGame
                && !others.is_empty()
            {
                div { class: "char-select-others",
                    p { class: "char-select-others-title", "Other players:" }
                    for (player, choice) in others {
                        div { class: "char-select-chosen-row",
                            span { class: "char-select-player-name", "{player}" }
                            span { class: if choice == "—" { "char-select-waiting" } else { "char-select-chosen-char" },
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
pub fn CharacterCardGrid(player_name: String, is_single_player: bool, universe: String) -> Element {
    let server_data = use_context::<Signal<ServerData>>();
    let all_characters = use_context::<Signal<Vec<Character>>>();

    let selected_names: Vec<String> = server_data()
        .core_game_data
        .heroes_chosen
        .iter()
        .filter(|(k, _)| {
            k.as_str() == player_name.as_str() || k.starts_with(&format!("{}__sp", player_name))
        })
        .map(|(_, v)| strip_id_suffix(v).to_string())
        .collect();

    let hero_chars: Vec<Character> = all_characters()
        .into_iter()
        .filter(|c| c.kind == lib_rpg::character_mod::character::CharacterKind::Hero)
        .filter(|c| universe.is_empty() || c.universe == universe)
        .collect();

    let extra_count = server_data()
        .core_game_data
        .heroes_chosen
        .keys()
        .filter(|k| k.starts_with(&format!("{}__sp", player_name)))
        .count();

    rsx! {
        div { class: "char-card-grid",
            for c in hero_chars {
                {
                    let is_selected = selected_names.contains(&c.db_full_name);
                    let taken_by = if !is_single_player {
                        server_data()
                            .core_game_data
                            .heroes_chosen
                            .iter()
                            .find(|(k, v)| {
                                k.as_str() != player_name.as_str()
                                    && !k.starts_with(&format!("{}__sp", player_name))
                                    && strip_id_suffix(v) == c.db_full_name.as_str()
                            })
                            .map(|(k, _)| k.clone())
                    } else {
                        None
                    };
                    let is_taken = taken_by.is_some();
                    let server_name = server_data().core_game_data.server_name.clone();
                    rsx! {
                        CharCardItem {
                            c: c.clone(),
                            player_name: player_name.clone(),
                            server_name,
                            is_single_player,
                            extra_count,
                            selected_names: selected_names.clone(),
                            is_selected,
                            is_taken,
                            taken_by,
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn CharCardItem(
    c: Character,
    player_name: String,
    server_name: String,
    is_single_player: bool,
    extra_count: usize,
    selected_names: Vec<String>,
    is_selected: bool,
    is_taken: bool,
    taken_by: Option<String>,
) -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let sd_signal = use_context::<Signal<ServerData>>();
    let max_hp = c.stats.all_stats.get(HP).map(|s| s.max).unwrap_or(0);
    let desc = c.description.clone();

    // onclick defined outside rsx! to prevent dx fmt corruption
    let cname = c.db_full_name.clone();
    let pname = player_name.clone();
    let sname = server_name.clone();
    let sel_snap = selected_names.clone();
    let onclick_handler = move |_: Event<MouseData>| {
        if is_taken {
            return;
        }
        let cn = cname.clone();
        let sn = sname.clone();
        let pn = pname.clone();
        let sel = sel_snap.clone();
        spawn(async move {
            if is_single_player {
                if sel.contains(&cn) {
                    let remove_key = sd_signal
                        .peek()
                        .core_game_data
                        .heroes_chosen
                        .iter()
                        .find(|(_, v)| strip_id_suffix(v) == cn.as_str())
                        .map(|(k, _)| k.clone());
                    if let Some(key) = remove_key {
                        let _ = socket
                            .send(ClientEvent::RemoveCharacterOnServerData(sn, key))
                            .await;
                    }
                    return;
                }
                let key = if sel.is_empty() {
                    pn.clone()
                } else {
                    format!("{}__sp{}", pn, extra_count + 1)
                };
                tracing::info!("SP: Adding {} under key {}", cn, key);
                let _ = socket
                    .send(ClientEvent::AddCharacterOnServerData(sn, key, cn))
                    .await;
            } else {
                tracing::info!("Selected character: {}", cn);
                let _ = socket
                    .send(ClientEvent::AddCharacterOnServerData(sn, pn, cn))
                    .await;
            }
        });
    };

    rsx! {
        div {
            class: if is_taken { "char-card char-card-taken" } else if is_selected { "char-card char-card-selected" } else { "char-card" },
            onclick: onclick_handler,
            div { class: "char-card-portrait",
                img {
                    src: format!("{}/{}.png", PATH_IMG, c.photo_name),
                    class: "char-card-img",
                    alt: "{c.db_full_name}",
                }
            }
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
            if is_taken {
                if let Some(taker) = taken_by.clone() {
                    div { class: "char-card-taken-label", "🔒 {taker}" }
                }
            } else if is_selected {
                div { class: "char-card-check", "✓" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::strip_id_suffix;

    #[test]
    fn strip_id_suffix_removes_hash_part() {
        assert_eq!(strip_id_suffix("Bulbasaur_#1"), "Bulbasaur");
        assert_eq!(strip_id_suffix("Mewtwo Armure_#1"), "Mewtwo Armure");
        assert_eq!(strip_id_suffix("Thraïn_#2"), "Thraïn");
    }

    #[test]
    fn strip_id_suffix_no_suffix_unchanged() {
        assert_eq!(strip_id_suffix("Charmander"), "Charmander");
        assert_eq!(strip_id_suffix(""), "");
    }

    #[test]
    fn strip_id_suffix_multiple_hashes_keeps_first_segment() {
        assert_eq!(strip_id_suffix("Hero_#1_#2"), "Hero");
    }
}

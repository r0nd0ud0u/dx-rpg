use std::collections::HashMap;

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use lib_rpg::{
    character_mod::character::Character,
    server::server_manager::{GamePhase, ServerData},
};

use crate::{
    components::{
        label::Label,
        select::{
            Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectOption,
            SelectTrigger, SelectValue,
        },
    },
    websocket_handler::event::{ClientEvent, ServerEvent},
};

#[component]
pub fn CharacterSelect() -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    // snapshot except local_login_name_session because it's used in the ClassSelect component
    let local_name = local_login_name_session();
    let server_data_snap = server_data();

    // avoid unexpected behavior for the select display
    if server_data_snap.players_data.players_info.is_empty() {
        return rsx! {};
    }
    // filter hashmap
    let players_except_current_client: HashMap<String, String> = server_data_snap
        .players_data
        .players_info
        .iter()
        .filter(|(k, _)| k.as_str() != local_name.as_str())
        .map(|(k, v)| {
            let name = v
                .character_names
                .first()
                .unwrap_or(&"No character selected".to_string())
                .split("_#")
                .next()
                .unwrap_or("No character selected")
                .to_string();
            (k.clone(), name)
        })
        .collect();

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
    rsx! {
        div { style: "display: flex; flex-direction: column; height: 40px; gap: 10px;",
            h3 { "Players:" }
            div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",
                if server_data_snap.core_game_data.game_phase == GamePhase::InitGame {
                    ClassSelect { player_name: local_login_name_session().clone() }
                } else {
                    div { style: "display: flex; flex-direction: column; height: 40px; gap: 10px;",
                        for player in connected.clone() {
                            Label { html_for: "sheet-demo-name", "{player.0}: {player.1} " }
                        }
                    }
                }
            }
            if server_data_snap.core_game_data.game_phase == GamePhase::InitGame {
                for player in players_except_current_client.clone() {
                    div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",
                        Label { html_for: "sheet-demo-name", "{player.0}" }
                        Label { html_for: "sheet-demo-name", "{player.1}" }
                    }
                }
            }
        }

    }
}

#[component]
pub fn ClassSelect(player_name: String) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let all_characters_names = use_context::<Signal<Vec<Character>>>();

    let characters = all_characters_names()
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
            let name = format!("{}-{}", c.class.to_emoji(), c.db_full_name);
            rsx! {
                SelectOption::<String> { index: i, value: name.to_string(), text_value: "{name}",
                    {name}
                    SelectItemIndicator {}
                }
            }
        });

    // callback for when the selected character changes
    let on_value_change_selected_character = move |e: Option<String>| {
        let l_player_name = player_name.clone();
        async move {
            match e {
                Some(value) => {
                    tracing::info!("Selected character: {}", value);
                    let db_full_name = value
                        .split_once("-")
                        .map(|(_, db_full_name)| db_full_name.to_string())
                        .unwrap_or_else(|| value.clone());
                    let _ = socket
                        .send(ClientEvent::AddCharacterOnServerData(
                            server_data().core_game_data.server_name.clone(),
                            l_player_name.clone(),
                            db_full_name.clone(),
                        ))
                        .await;
                }
                None => {
                    tracing::info!("No character selected");
                }
            }
        }
    };

    // render the select component
    rsx! {

        Select::<String> {
            placeholder: "Select your character",
            on_value_change: on_value_change_selected_character,
            SelectTrigger { aria_label: "Select Trigger", width: "12rem", SelectValue {} }
            SelectList { aria_label: "Select Demo",
                SelectGroup {
                    SelectGroupLabel { "Characters" }
                    {characters}
                }
            }
        }
    }
}

use std::collections::HashMap;

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};

use crate::{
    components::{
        label::Label,
        select::{
            Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectOption,
            SelectTrigger, SelectValue,
        },
    },
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        game_state::{GamePhase, PlayerInfo, ServerData},
    },
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
    if server_data_snap.players_info.is_empty() {
        return rsx! {};
    }
    // filter hashmap
    let players_except_current_client: HashMap<String, PlayerInfo> = server_data_snap
        .players_info
        .iter()
        .filter(|(k, _)| k.as_str() != local_name.as_str())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let connected: HashMap<String, String> = server_data_snap
        .app
        .heroes_chosen
        .iter()
        .map(|(key, value)| {
            let status = if server_data_snap.players_info.contains_key(key) {
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
                if server_data_snap.app.game_phase == GamePhase::InitGame {
                    ClassSelect { player_name: local_login_name_session().clone() }
                } else {
                    div { style: "display: flex; flex-direction: column; height: 40px; gap: 10px;",
                        for player in connected.clone() {

                            Label { html_for: "sheet-demo-name", "{player.0}: {player.1} " }
                        }
                    }
                }
            }
            if server_data_snap.app.game_phase == GamePhase::InitGame {
                for player in players_except_current_client.clone() {
                    div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",
                        Label { html_for: "sheet-demo-name", "{player.0}" }
                        Label { html_for: "sheet-demo-name",
                            "{player.1.character_names.get(0).unwrap_or(&\"No character selected\".to_string())}"
                        }
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

    // get character name for the player
    let character_name = server_data()
        .players_info
        .get(&player_name)
        .and_then(|player_info| player_info.character_names.first().cloned())
        .unwrap_or_else(|| "Select your character".to_string());
    tracing::trace!(
        "Character name for player {}: {}",
        player_name,
        character_name
    );

    let data = server_data();

    let players_name: Vec<String> = data
        .app
        .game_manager
        .pm
        .all_heroes
        .iter()
        .map(|h| h.name.clone())
        .collect();

    let characters = players_name.into_iter().enumerate().map(|(i, c)| {
        let c = c.as_str();
        rsx! {
            SelectOption::<String> { index: i, value: c.to_string(), text_value: "{c}",
                {
                    // TODO: use actual icons for each character
                    format!(
                        "{} {c}",
                        match c {
                            "Warrior" => "🗡️",
                            "Mage" => "🪄",
                            "Rogue" => "🗡️",
                            _ => "",
                        },
                    )
                }
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
                    let _ = socket
                        .send(ClientEvent::AddCharacterOnServerData(
                            server_data().app.server_name.clone(),
                            l_player_name.clone(),
                            value.clone(),
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
            default_value: character_name.clone(),
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

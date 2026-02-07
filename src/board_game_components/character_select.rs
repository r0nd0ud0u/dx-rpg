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
        game_state::ServerData,
    },
};

#[component]
pub fn CharacterSelect() -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    // snapshot
    let snap_players_list = server_data()
        .players
        .keys()
        .cloned()
        .collect::<Vec<String>>();
    rsx! {
        div { style: "display: flex; flex-direction: column; height: 40px; gap: 10px;",
            div { "Players:" }
            for player in snap_players_list {
                div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",
                    Label { html_for: "sheet-demo-name", "{player}" }
                    ClassSelect {}
                }
            }

        }

    }
}

#[component]
pub fn ClassSelect() -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let mut selected_value = use_signal(String::new);
    let players_name = server_data()
        .app
        .game_manager
        .pm
        .all_heroes
        .iter()
        .map(|h| h.name.clone())
        .collect::<Vec<String>>();
    let characters = players_name.into_iter().enumerate().map(|(i, c)| {
        let c = c.as_str();
        rsx! {
            SelectOption::<String> { index: i, value: c.to_string(), text_value: "{c}",
                {
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

    let on_value_change_selected_character = move |e: Option<String>| async move {
        match e {
            Some(value) => {
                tracing::info!("Selected character: {}", value);
                let _ = socket
                    .send(ClientEvent::AddCharacterOnServerData(
                        server_data().app.server_name.clone(),
                        local_login_name_session(),
                        value.clone(),
                    ))
                    .await;
                selected_value.set(value)
            }
            None => {
                tracing::info!("No character selected");
                selected_value.set("".to_string())
            }
        }
    };
    rsx! {

        Select::<String> {
            placeholder: "Select a character...",
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

use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus_fullstack::UseWebsocket;
use rand::Rng;
use wasm_bindgen_futures::spawn_local;

use crate::websocket_components::{
    app::{PLAYER_NAME, PLAYER_NAME_PENDING},
    event::{ClientEvent, ServerEvent},
};

pub fn random_name() -> String {
    const NOUNS: [&str; 4] = ["Thalia", "Elara", "Thrain", "Azrak"];
    let mut rng = rand::rng();
    let noun = NOUNS.get(rng.random_range(0..=4)).unwrap();
    noun.to_string()
}

#[component]
pub fn PlayerName() -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    // Snapshot of the current global name for initial value
    let current_global_name = PLAYER_NAME.read();
    // Local input state
    let mut input = use_signal(|| current_global_name.clone());
    // snapshot it for this render
    let pending = *PLAYER_NAME_PENDING.read();

    // Pull the input value for use in the RSX (so interpolation works).
    let input_value = input();

    rsx! {
        div { style: "margin:8px 0; padding:8px; border:1px solid #ccc; display:flex; gap:8px; align-items:center;",
            p { "Choose a player name:" }
            input {
                name: "player_name",
                placeholder: "Enter your name here",
                value: "{input_value}",
                oninput: move |e| {
                    *input.write() = e.value();
                },
            }
            button {
                disabled: input_value.trim().is_empty() || pending,
                onclick: move |_| {
                    if !input.read().trim().is_empty() && !pending {
                        let new_name = input.read().to_string();
                        // mark pending so the UI can disable further renames until confirmation
                        *PLAYER_NAME_PENDING.write() = true;
                        // send SetName to server asynchronously; wait for server to send NameAccepted
                        spawn_local(async move {
                            let _ = socket.clone().send(ClientEvent::SetName(new_name)).await;
                        });
                    }
                },
                "Rename"
            }
        }
    }
}

use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus::{fullstack::UseWebsocket, logger::tracing};
use dioxus_primitives::scroll_area::ScrollDirection;

use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::{
    application,
    common::Route,
    components::{
        button::{Button, ButtonVariant},
        scroll_area::ScrollArea,
    },
};

#[component]
pub fn LoadGame() -> Element {
    // contexts
    let games_list = use_context::<Signal<Vec<std::path::PathBuf>>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    // states
    let mut active_button: Signal<i64> = use_signal(|| -1);
    let navigator = use_navigator();

    rsx! {
        div { class: "home-container",
            h4 { "Load game (games saved: {games_list().len()})" }
            ScrollArea {
                width: "25em",
                height: "10em",
                border: "1px solid var(--primary-color-6)",
                border_radius: "0.5em",
                padding: "0 1em 1em 1em",
                direction: ScrollDirection::Vertical,
                tabindex: "0",
                div { class: "scroll-content",
                    for (index , i) in games_list.read().iter().enumerate() {
                        Button {
                            variant: if active_button() as usize == index { ButtonVariant::Destructive } else { ButtonVariant::Primary },
                            disabled: active_button() == index as i64,
                            onclick: move |_| async move { active_button.set(index as i64) },
                            "{i.clone().to_string_lossy()}"
                        }
                    }
                }
            }

            Button {
                variant: ButtonVariant::Secondary,
                disabled: active_button() == -1,
                onclick: move |_| {
                    let cur_game = games_list
                        .read()
                        .get(active_button() as usize)
                        .unwrap()
                        .to_owned();
                    async move {
                        let _ = socket
                            .clone()
                            .send(
                                ClientEvent::LoadGame(cur_game.clone(), local_login_name_session()),
                            )
                            .await;
                        navigator.push(Route::LobbyPage {});
                    }
                },
                "Valid"
            }

            Button {
                variant: ButtonVariant::Secondary,
                disabled: active_button() == -1,
                onclick: move |_| {
                    let cur_game = games_list
                        .read()
                        .get(active_button() as usize)
                        .unwrap()
                        .to_owned();
                    async move {
                        match application::delete_game(cur_game.clone()).await {
                            Ok(_) => {
                                let _ = socket
                                    .clone()
                                    .send(
                                        ClientEvent::RequestSavedGameList(
                                            local_login_name_session().clone(),
                                        ),
                                    )
                                    .await;
                            }
                            Err(e) => {
                                tracing::error!("Error deleting game: {}", e);
                                return;
                            }
                        };
                        active_button.set(-1);
                    }
                },
                "Delete Game"
            }
        }
    }
}

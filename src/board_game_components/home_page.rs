use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_i18n::t;
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::{
    board_game_components::login_page::LoginPage,
    common::{DISCONNECTED_USER, Route},
    websocket_handler::event::{ClientEvent, ServerEvent},
};

/// Home page
#[component]
pub fn Home() -> Element {
    // contexts
    let local_login_name_session = use_context::<Signal<String>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>;
    // Snapshot for this render
    let user_name = local_login_name_session();

    if user_name == *DISCONNECTED_USER {
        rsx! {
            LoginPage {}
        }
    } else {
        rsx! {
            div { class: "home-container",
                div { class: "rotate-scale-up",
                    h1 { class: "rpg-title", {t!("home-title")} }
                }
                p { class: "rpg-subtitle", {t!("home-welcome", user_name: user_name.clone())} }
                div { class: "action-grid",
                    div { class: "action-card",
                        span { class: "action-icon", "🏰" }
                        Link {
                            class: "header-text",
                            to: Route::CreateServer {},
                            onclick: move |_| {
                                server_data().write().core_game_data.game_phase = GamePhase::Default;
                            },
                            {t!("home-create-server")}
                        }
                        p { class: "action-desc", {t!("home-create-server-desc")} }
                    }
                    div { class: "action-card",
                        span { class: "action-icon", "🗺️" }
                        Link {
                            class: "header-text",
                            to: Route::JoinOngoingGame {},
                            onclick: move |_| {
                                async move {
                                    let _ = socket.send(ClientEvent::RequestOnGoingGamesList).await;
                                }
                            },
                            {t!("home-join-game")}
                        }
                        p { class: "action-desc", {t!("home-join-game-desc")} }
                    }
                }
            }
        }
    }
}

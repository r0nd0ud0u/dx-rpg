use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};

use crate::{
    application::Application,
    auth_manager::server_fn::get_user_name,
    board_game_components::common_comp::ButtonLink,
    common::{Route, SERVER_NAME},
    websocket_handler::event::{ClientEvent, ServerEvent},
};

#[component]
pub fn LobbyPage() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let app = use_context::<Signal<Application>>();

    // send message to server to create a new game
    use_effect(move || {
        spawn(async move {
            let name = match get_user_name().await {
                Ok(name) => name,
                Err(_) => "".to_string(),
            };
            if name.is_empty() {
                tracing::info!("User name is empty, cannot create new game");
                return;
            }
            // TODO set server name based on user name + random string
            *SERVER_NAME.write() = name.clone();
            let _ = socket.send(ClientEvent::StartGame(name)).await;
        });
    });

    rsx! {
        div { class: "home-container",
            h1 { "LobbyPage" }
            if app.read().is_game_running {
                ButtonLink {
                    target: Route::StartGamePage {}.into(),
                    name: "Start Game".to_string(),
                }
            }
        }
    }
}

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
};

use crate::{
    auth_manager::server_fn::get_user_name,
    common::SERVER_NAME,
    websocket_handler::event::{ClientEvent, ServerEvent},
};

pub async fn send_initialize_game(
    user_name: &str,
    socket: UseWebsocket<ClientEvent, ServerEvent, CborEncoding>,
) {
    if user_name.is_empty() {
        tracing::info!("User name is empty, cannot create new game");
        return;
    }
    // TODO set server name based on user name + random string
    *SERVER_NAME.write() = user_name.to_string();
    tracing::info!("Sending InitializeGame with server name: {}", SERVER_NAME());
    let _ = socket
        .send(ClientEvent::InitializeGame(
            SERVER_NAME().clone(),
            user_name.to_string(),
        ))
        .await;
}

pub async fn send_start_game(socket: UseWebsocket<ClientEvent, ServerEvent, CborEncoding>) {
    let _ = socket
        .send(ClientEvent::StartGame(SERVER_NAME().clone()))
        .await;
}

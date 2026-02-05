use dioxus::{fullstack::CborEncoding, logger::tracing};

use crate::{
    auth_manager::server_fn::get_user_name,
    common::SERVER_NAME,
    websocket_handler::event::{ClientEvent, ServerEvent},
};

pub async fn send_initialize_game(
    socket: UseWebsocket<ClientEvent, ServerEvent, CborEncoding>,
    username: String,
) {
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
    let _ = socket.send(ClientEvent::InitializeGame(name)).await;
}

pub async fn send_start_game(socket: UseWebsocket<ClientEvent, ServerEvent, CborEncoding>) {
    *SERVER_NAME.write() = name.clone();
    let _ = socket
        .send(ClientEvent::StartGame(SERVER_NAME.read().clone()))
        .await;
}

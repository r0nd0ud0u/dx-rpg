use dioxus::logger::tracing;

use crate::{
    common::SERVER_NAME, game_channel::GameChannel, websocket_handler::event::ClientEvent,
};

pub async fn send_initialize_game(
    user_name: &str,
    universe: &str,
    is_single_player: bool,
    socket: GameChannel,
) {
    if user_name.is_empty() {
        tracing::info!("User name is empty, cannot create new game");
        return;
    }
    *SERVER_NAME.write() = user_name.to_string();
    tracing::info!("Sending InitializeGame with server name: {}", SERVER_NAME());
    let _ = socket
        .send(ClientEvent::InitializeGame(
            SERVER_NAME().clone(),
            user_name.to_string(),
            universe.to_string(),
            is_single_player,
        ))
        .await;
}

pub async fn send_start_game(socket: GameChannel) {
    let _ = socket
        .send(ClientEvent::StartGame(SERVER_NAME().clone()))
        .await;
}

pub async fn request_save_game(socket: GameChannel, player_name: &str) {
    let _ = socket
        .send(ClientEvent::SaveGame(
            SERVER_NAME().clone(),
            player_name.to_owned(),
        ))
        .await;
}

pub async fn send_join_server_data(socket: GameChannel, server_name: &str, player_name: &str) {
    let _ = socket
        .send(ClientEvent::JoinServerData(
            server_name.to_string(),
            player_name.to_string(),
        ))
        .await;
}

pub async fn request_update_saved_game_list_display(socket: GameChannel, player_name: &str) {
    let _ = socket
        .send(ClientEvent::RequestSavedGameList(player_name.to_owned()))
        .await;
}

pub async fn send_disconnect_from_server_data(socket: GameChannel, player_name: &str) {
    let _ = socket
        .send(ClientEvent::DisconnectFromServerData(
            SERVER_NAME().clone(),
            player_name.to_string(),
        ))
        .await;
}

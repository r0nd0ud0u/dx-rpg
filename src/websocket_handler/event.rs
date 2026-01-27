use dioxus::fullstack::{CborEncoding, WebSocketOptions, Websocket};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

#[cfg(feature = "server")]
use once_cell::sync::Lazy;

#[cfg(feature = "server")]
use tokio::sync::mpsc;

use crate::application::Application;
use crate::websocket_handler::game_state::GameStateWebsocket;
use dioxus::logger::tracing;

#[cfg(feature = "server")]
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

/// server only: map of client id -> sender to that client's outgoing queue
#[cfg(feature = "server")]
static CLIENTS: Lazy<Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<ServerEvent>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// server only: shared game state
#[cfg(feature = "server")]
static GAME_STATE: Lazy<Arc<Mutex<GameStateWebsocket>>> =
    Lazy::new(|| Arc::new(Mutex::new(GameStateWebsocket::default())));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientEvent {
    SetName(String),
    Disconnect(String),
    StartGame(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    Message(String),
    AssignPlayerId(u32),
    SnapshotPlayers(GameStateWebsocket),
    UpdateApplication(Application),
}

#[get("/api/new-event")]
pub async fn new_event(
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |mut socket| async move {
        #[cfg(feature = "server")]
        {
            // Assign id
            let id = NEXT_CLIENT_ID.fetch_add(1, Ordering::SeqCst) as u32;

            // Channel for outgoing messages to this client
            let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

            // Register client
            {
                let mut clients = CLIENTS.lock().unwrap();
                clients.insert(id as usize, tx);
                println!("Client {} connected (total: {})", id, clients.len());
            }

            let _ = socket.send(ServerEvent::AssignPlayerId(id)).await;
            let _ = socket.send(ServerEvent::Message(format!("Welcome! (id={})", id))).await;

            // Main loop: handle incoming socket messages and outgoing queued messages
            loop {
                use dioxus::logger::tracing;

                tokio::select! {
                    // Outgoing messages destined for this client
                    maybe_msg = rx.recv() => {
                        if let Some(msg) = maybe_msg {
                            let _ = socket.send(msg).await;
                        } else {
                            break;
                        }
                    },

                    // Incoming message from client
                    res = socket.recv() => {
                        match res {
                            Ok(ClientEvent::SetName(name)) => {
                                println!("Received set_name request from client {}: {:?}", id, name);
                                add_player(name);
                            }
                            Ok(ClientEvent::Disconnect(name)) => {
                                tracing::info!("{} is disconnected", name);
                                send_disconnection_to_server(name);
                            }
                            Ok(ClientEvent::StartGame(name)) => {
                                tracing::info!("{} is starting a new game", name);
                                create_new_game_by_player(name).await;
                            }
                            Err(_) => {
                                println!("Client {} disconnected", id);
                                break;
                            }
                        }
                    }
                }
            }

            // cleanup on disconnect
            {
                let mut clients = CLIENTS.lock().unwrap();
                clients.remove(&(id as usize));
                println!("Client {} removed. Remaining: {}", id, clients.len());
            }
        }
    }))
}

#[cfg(feature = "server")]
pub fn add_player(name: String) {
    let mut map = GAME_STATE.lock().unwrap();
    map.players.push(name);
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::SnapshotPlayers(map.clone()));
    }
}

#[cfg(feature = "server")]
pub fn send_disconnection_to_server(name: String) {
    let mut map = GAME_STATE.lock().unwrap();
    map.players.retain(|player| *player != name);
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::SnapshotPlayers(map.clone()));
    }
}

#[cfg(feature = "server")]
pub async fn create_new_game_by_player(name: String) {
    use crate::application;

    let mut map = GAME_STATE.lock().unwrap();

    // add new ongoing game
    match application::try_new().await {
        Ok(app) => {
            // start a new game
            app.game_manager.start_new_game();
            let _ = app.game_manager.start_new_turn();
            // update ongoing games status
            let all_games_dir = format!(
                "{}/ongoing-games.json",
                app.game_manager.game_paths.games_dir.to_string_lossy()
            );
            // add the current game directory to ongoing games
            map.ongoing_games
                .push(app.game_manager.game_paths.current_game_dir.clone());
            match application::save(
                all_games_dir,
                serde_json::to_string_pretty(&map.ongoing_games).unwrap(),
            )
            .await
            {
                Ok(_) => tracing::info!("Game state saved successfully"),
                Err(e) => tracing::error!("Failed to save game state: {}", e),
            }
            // save the game manager state
            let path = format!(
                "{}",
                &app.game_manager
                    .game_paths
                    .current_game_dir
                    .join("game_manager.json")
                    .to_string_lossy(),
            );
            match application::create_dir(app.game_manager.game_paths.current_game_dir.clone())
                .await
            {
                Ok(()) => tracing::info!("Directory created successfully"),
                Err(e) => tracing::error!("Failed to create directory: {}", e),
            }
            match application::save(
                path.to_owned(),
                serde_json::to_string_pretty(&app.game_manager.clone()).unwrap(),
            )
            .await
            {
                Ok(()) => tracing::info!("Game manager state saved successfully"),
                Err(e) => tracing::error!("Failed to save game manager state: {}", e),
            }
        }
        Err(_) => tracing::error!("no app"),
    }

    app.is_game_running = true;
    // update for all clients
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::SnapshotPlayers(map.clone()));
        let _ = sender.send(ServerEvent::UpdateApplication(app.clone()));
    }
}

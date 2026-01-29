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
use crate::websocket_handler::game_state::GameStateManager;
use dioxus::logger::tracing;

#[cfg(feature = "server")]
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

/// server only: map of client id -> sender to that client's outgoing queue
#[cfg(feature = "server")]
static CLIENTS: Lazy<Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<ServerEvent>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// server only: shared game state
#[cfg(feature = "server")]
static GAMES_MANAGER: Lazy<Arc<Mutex<GameStateManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(GameStateManager::default())));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientEvent {
    SetName(String),
    Disconnect(String),
    StartGame(String),
    LaunchAttack(String, String), // `String`: server_name, `String`: atk name
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    Message(String),
    AssignPlayerId(u32),
    SnapshotPlayers(GameStateManager),
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
                use crate::auth_manager::server_fn::get_user_name;

                let mut clients = CLIENTS.lock().unwrap();
                clients.insert(id as usize, tx);
                tracing::info!("Client {} connected (total: {})", id, clients.len());
                // Try potential reconnection
                let username = get_user_name().await.unwrap_or_default();
                if !username.is_empty() {
                    add_player(username, id);
                }
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
                        tracing::info!("Receiving message from client {}, message: {:?}", id, res);
                        match res {
                            Ok(ClientEvent::SetName(name)) => {
                                tracing::info!("Received set_name request from client {}: {:?}", id, name);
                                add_player(name, id);
                            }
                            Ok(ClientEvent::Disconnect(name)) => {
                                tracing::info!("{} is disconnected", name);
                                send_disconnection_to_server(name);
                            }
                            Ok(ClientEvent::StartGame(name)) => {
                                tracing::info!("{} is starting a new game", name);
                                create_new_game_by_player(name, id).await;
                            }
                            Ok(ClientEvent::LaunchAttack(server_name, selected_atk)) => {
                                tracing::info!("A new atk has been launched");
                                update_app_after_atk(server_name, selected_atk);
                            }
                            Err(_) => {
                                tracing::info!("Client {} disconnected", id);
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
                tracing::info!("Client {} removed. Remaining: {}", id, clients.len());
            }
        }
    }))
}

#[cfg(feature = "server")]
pub fn add_player(name: String, id: u32) {
    let mut map = GAMES_MANAGER.lock().unwrap();
    map.players.insert(name, id);
    /* let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::SnapshotPlayers(map.clone()));
    } */
}

#[cfg(feature = "server")]
pub fn send_disconnection_to_server(name: String) {
    let mut map = GAMES_MANAGER.lock().unwrap();
    map.players.retain(|player_name, _| player_name != &name);
    /*     let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::SnapshotPlayers(map.clone()));
    } */
}

#[cfg(feature = "server")]
pub async fn create_new_game_by_player(name: String, id: u32) {
    use crate::application;
    let mut gm = GAMES_MANAGER.lock().unwrap();

    // add new ongoing game
    match application::try_new().await {
        Ok(mut app) => {
            tracing::info!("New application created for player: {}", name);
            // start a new game
            use crate::websocket_handler::game_state::ServerData;
            app.game_manager.start_new_game();
            let _ = app.game_manager.start_new_turn();
            // update ongoing games status
            let all_games_dir = format!(
                "{}/ongoing-games.json",
                app.game_manager.game_paths.games_dir.to_string_lossy()
            );
            // add the current game directory to ongoing games
            gm.ongoing_games_path
                .push(app.game_manager.game_paths.current_game_dir.clone());
            match application::save(
                all_games_dir,
                serde_json::to_string_pretty(&gm.ongoing_games_path).unwrap(),
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
            app.is_game_running = true;
            // name of the server
            // TODO set server name based on user name + random string
            app.server_name = name.clone();
            // add to ongoing games map
            gm.servers_data.insert(
                name.clone(),
                ServerData {
                    app: app.clone(),
                    players: HashMap::from([(name.clone(), id)]),
                },
            );
            tracing::info!("servers data keys: {:?}", gm.servers_data.keys());
            // update for the clients connected to that server
            drop(gm);
            update_clients_app(name.clone(), app.clone());
        }
        Err(_) => tracing::error!("no app"),
    }
}

#[cfg(feature = "server")]
fn update_clients_app(server_name: String, app: Application) {
    let mut gm = GAMES_MANAGER.lock().unwrap();
    // update the app in the game state manager
    let server_data = match gm.servers_data.get_mut(server_name.as_str()) {
        Some(server_data) => {
            server_data.app = app.clone();
            server_data
        }
        None => {
            tracing::info!("no server data for server: {}", server_name);
            return;
        }
    };
    tracing::info!(
        "Clients ids: {:?} for server: {}",
        server_data.players.values(),
        server_name
    );
    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if server_data
            .players
            .values()
            .any(|&id| id == other_id as u32)
        {
            tracing::info!("Sending update to client id: {}", other_id);
            let _ = sender.send(ServerEvent::UpdateApplication(app.clone()));
        }
    }
}

#[cfg(feature = "server")]
pub fn update_app_after_atk(server_name: String, selected_atk_name: String) {
    // get app by server name
    let mut gm = GAMES_MANAGER.lock().unwrap();
    let mut app = match gm.servers_data.get(&server_name) {
        Some(server_data) => server_data.app.clone(),
        None => {
            tracing::error!("No application found for server name: {}", server_name);
            return;
        }
    };
    drop(gm);
    // launch attack
    let _ = app.game_manager.launch_attack(&selected_atk_name);
    // update clients
    update_clients_app(server_name, app.clone());
}

#[cfg(feature = "server")]
pub fn reconnection_player(name: String, id: u32) {
    let mut gm = GAMES_MANAGER.lock().unwrap();
    for (server_name, server_data) in gm.servers_data.iter_mut() {
        if server_data.players.contains_key(&name) {
            // update player's id
            server_data.players.insert(name.clone(), id);
            // update client app
            update_clients_app(
                name.clone(),
                gm.servers_data.get(&name).unwrap().app.clone(),
            );
            tracing::info!("Player {} reconnected with id {}", name, id);
            break;
        }
    }
}

#[cfg(feature = "server")]
use crate::application::init_application;
#[cfg(feature = "server")]
use crate::application::save_on_going_games;
use crate::websocket_handler::game_state::OnGoingGame;
use crate::websocket_handler::game_state::ServerData;
use async_std::task::sleep;
use dioxus::fullstack::{CborEncoding, WebSocketOptions, Websocket};
use dioxus::logger::tracing;
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

use crate::application::{self, Application};
use crate::websocket_handler::game_state::GameStateManager;

#[cfg(feature = "server")]
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

/// server only: map of client id -> sender to that client's outgoing queue
#[cfg(feature = "server")]
type ClientId = usize;
#[cfg(feature = "server")]
type ClientTx = mpsc::UnboundedSender<ServerEvent>;
#[cfg(feature = "server")]
type ClientsMap = HashMap<ClientId, ClientTx>;
#[cfg(feature = "server")]
type SharedClients = Arc<Mutex<ClientsMap>>;
#[cfg(feature = "server")]
static CLIENTS: Lazy<SharedClients> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// server only: shared game state
#[cfg(feature = "server")]
static GAMES_MANAGER: Lazy<Arc<Mutex<GameStateManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(GameStateManager::default())));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientEvent {
    LoginAllSessions(String, i64),  // `String`: username, `i64`: sql-id
    LogOut(String),                 // `String`: username
    InitializeGame(String, String), // `String`: server_name, `String`: player_name
    AddCharacterOnServerData(String, String, String), // `String`: server_name, `String`: player_name, `String`: character_name
    StartGame(String),                                // `String`: server_name
    LaunchAttack(String, String),                     // `String`: server_name, `String`: atk name
    AddPlayer(String),                                // `String`: username
    JoinServerData(String, String), // `String`: server_name, `String`: player_name
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    NewClientOnExistingPlayer(String, u32), // welcome message, player id
    AssignPlayerId(u32),                    // player id
    UpdateApplication(Box<Application>),
    ReconnectAllSessions(String, i64), // username, sql-id
    UpdateServerData(Box<ServerData>), // server data
    UpdateOngoingGames(Vec<OnGoingGame>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerOwnEvent {
    AutoAtkIsDone(String), // servername
}

#[get("/api/new-event")]
pub async fn on_rcv_client_event(
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |mut socket| async move {
        #[cfg(feature = "server")]
        {
            // Assign id
            let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::SeqCst) as u32;

            // Channel for outgoing messages to this client
            let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();
            // channel between main-thread server and another spawn
            let (tx_server, mut rx_server) = mpsc::unbounded_channel::<ServerOwnEvent>();

            // Register client
            {
                let mut clients = CLIENTS.lock().unwrap();
                clients.insert(client_id as usize, tx);
                tracing::info!("Client {} connected (total: {})", client_id, clients.len());
            }

            let _ = socket.send(ServerEvent::AssignPlayerId(client_id)).await;
            let _ = socket.send(ServerEvent::NewClientOnExistingPlayer(format!("Welcome! (id={})", client_id), client_id)).await;


            // Main loop: handle incoming socket messages and outgoing queued messages
            loop {
                tokio::select! {
                    // Outgoing messages destined for this client
                    maybe_msg = rx.recv() => {
                        if let Some(msg) = maybe_msg {
                            let _ = socket.send(msg).await;
                        } else {
                            break;
                        }
                    },

                    maybe_server_msg = rx_server.recv() => {
                        tracing::info!("Receiving message from other-thread server {}, message: {:?}", client_id, maybe_server_msg);
                        match maybe_server_msg {
                            Some(ServerOwnEvent::AutoAtkIsDone(server_name)) => {
                                if get_app_by_server_name(&server_name).is_some(){
                                    update_app_after_atk(&server_name, None);
                                }
                            }
                            None => {}
                        }
                    },

                    // Incoming message from client
                    res = socket.recv() => {
                        tracing::info!("Receiving message from client {}, message: {:?}", client_id, res);
                        match res {
                            Ok(ClientEvent::LoginAllSessions(username, sql_id)) => {
                                tracing::info!("Received set_name request from client {}: {:?}", sql_id, username);
                                login_all_sessions(username, sql_id);
                            }
                            Ok(ClientEvent::AddPlayer(username)) => {
                                tracing::info!("Adding new player from client {}: {:?}", client_id, username);
                                add_player(username, client_id);
                            }
                            Ok(ClientEvent::LogOut(user_name)) => {
                                tracing::info!("{} is logged out", user_name);
                                send_logout_to_server(user_name);
                            }
                            Ok(ClientEvent::StartGame(server_name)) => {
                                tracing::info!("{} is starting a new game", server_name);
                                start_new_game_by_player(&server_name).await;
                            }
                            Ok(ClientEvent::InitializeGame(server_name, player_name)) => {
                                tracing::info!("{} is initializing a new game", server_name);
                                init_new_game_by_player(&server_name, client_id, &player_name).await;
                                update_clients_server_data(&server_name);
                            }
                            Ok(ClientEvent::AddCharacterOnServerData(server_name, player_name, character_name)) => {
                                tracing::info!("{} is adding character {} to server data", player_name, character_name);
                                add_character_on_server_data(&server_name, &player_name, &character_name);
                                update_clients_server_data(&server_name);
                            }
                            Ok(ClientEvent::LaunchAttack(server_name, selected_atk)) => {
                                tracing::info!("A new atk has been launched");
                                update_app_after_atk(&server_name, Some(&selected_atk));
                                update_clients_server_data(&server_name);
                                // is ennemy turn ? 
                                process_ennemy_atk(&server_name, tx_server.clone()).await;
                            }
                            Ok(ClientEvent::JoinServerData(server_name, player_name)) => {
                                tracing::info!("Player {} with id {} is joining server data for server {}", player_name, client_id, server_name);
                                update_lobby_page_after_joining_game(&server_name, &player_name, client_id);
                            }
                            Err(_) => {
                                // ClientEvent::ConnectionClosed
                                tracing::info!("Client {} disconnected", client_id);
                                // TODO get server name from GAMES_MANAGER by player id
                                send_disconnection_to_server(client_id).await;
                                break;
                            }
                        }
                    }
                }
            }

            // cleanup on disconnect
            {
                let mut clients = CLIENTS.lock().unwrap();
                clients.remove(&(client_id as usize));
                tracing::info!("Client {} removed. Remaining: {}", client_id, clients.len());
            }
        }
    }))
}

#[cfg(feature = "server")]
pub fn add_player(name: String, id: u32) {
    let mut gm = GAMES_MANAGER.lock().unwrap();
    gm.players.entry(name.clone()).or_default().push(id);
    tracing::info!("All connected players: {:?}", gm.players);
}

#[cfg(feature = "server")]
pub fn login_all_sessions(username: String, sql_id: i64) {
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::ReconnectAllSessions(username.clone(), sql_id));
    }
}

#[cfg(feature = "server")]
pub fn send_logout_to_server(user_name: String) {
    let mut gm = GAMES_MANAGER.lock().unwrap();
    // remove player from GameStateManager
    gm.players
        .retain(|player_name, _| player_name != &user_name);
    // remove from servers data
    if let Some(server_data) = gm.servers_data.get_mut(&user_name) {
        server_data
            .players_info
            .retain(|player_name, _| player_name != &user_name);
    }
    tracing::info!("All connected players after logout: {:?}", gm.players);
}

#[cfg(feature = "server")]
pub async fn send_disconnection_to_server(cur_player_id: u32) {
    let mut gm = GAMES_MANAGER.lock().unwrap();
    let username = gm
        .players
        .iter()
        .find_map(|(player_name, ids)| {
            if ids.contains(&cur_player_id) {
                Some(player_name.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Unknown".to_string());
    tracing::info!(
        "Player {} with id {} is disconnecting",
        username,
        cur_player_id
    );
    // remove player id from GameStateManager
    gm.players.retain(|player_name, ids| {
        if player_name == &username {
            ids.retain(|&id| id != cur_player_id);
            !ids.is_empty()
        } else {
            true
        }
    });
    tracing::info!(
        "All connected players after disconnection: {:?}",
        gm.players
    );
    // remove from servers data
    if let Some(server_data) = gm.servers_data.get_mut(&username) {
        server_data.players_info.retain(|player_name, pl| {
            if player_name == &username {
                pl.player_ids.retain(|&id| id != cur_player_id);
                !pl.player_ids.is_empty()
            } else {
                true
            }
        });
    }
}

#[cfg(feature = "server")]
pub async fn start_new_game_by_player(server_name: &str) {
    // update app state
    let mut gm = GAMES_MANAGER.lock().unwrap();
    if let Some(server_data) = gm.servers_data.get_mut(server_name) {
        server_data.app.game_manager.start_game();
        // update app state
        server_data.app.is_game_running = true;
    }
    drop(gm);
    update_clients_app(
        server_name,
        &get_app_by_server_name(server_name).unwrap_or_default(),
    );
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
pub async fn init_new_game_by_player(server_name: &str, id: u32, player_name: &str) {
    // add new ongoing game
    match Application::try_new().await {
        Ok(mut app) => {
            tracing::info!("New application created for player: {}", server_name);
            // init a new game
            init_application(server_name, &mut app);
            // update ongoing games status
            save_on_going_games(&app)
                .await
                .unwrap_or_else(|e| tracing::error!("Failed to update ongoing games: {}", e));
            // save the game manager state
            save_game_manager_state(&app).await;
            // update ongoing servers data list
            let mut gm: std::sync::MutexGuard<'_, GameStateManager> = GAMES_MANAGER.lock().unwrap();
            gm.ongoing_games.push(OnGoingGame {
                path: app.game_manager.game_paths.current_game_dir.clone(),
                server_name: server_name.to_string(),
            });
            drop(gm);
            // add server data
            add_server_data_with_player(&app, server_name, id, player_name);
            // update for the clients connected to that server
            update_clients_app(server_name, &app);
            update_clients_server_data(server_name);
            update_clients_ongoing_games();
        }
        Err(_) => tracing::error!("no app"),
    }
}

#[cfg(feature = "server")]
async fn save_game_manager_state(app: &Application) {
    let path = format!(
        "{}",
        &app.game_manager
            .game_paths
            .current_game_dir
            .join("game_manager.json")
            .to_string_lossy(),
    );
    match application::create_dir(app.game_manager.game_paths.current_game_dir.clone()).await {
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

#[cfg(feature = "server")]
fn update_clients_app(server_name: &str, app: &Application) {
    let mut gm = GAMES_MANAGER.lock().unwrap();
    // update the app in the game state manager
    let server_data = match gm.servers_data.get_mut(server_name) {
        Some(server_data) => {
            server_data.app = app.clone();
            server_data
        }
        None => {
            tracing::info!("no server data for server: {}", server_name);
            return;
        }
    };
    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if server_data
            .players_info
            .values()
            .any(|player_info| player_info.player_ids.contains(&(other_id as u32)))
        {
            let _ = sender.send(ServerEvent::UpdateApplication(Box::new(app.clone())));
        }
    }
}

#[cfg(feature = "server")]
fn update_clients_server_data(server_name: &str) {
    let server_data = match get_server_data_by_server_name(server_name) {
        Some(server_data) => {
            tracing::info!(
                "Clients ids: {:?} for server: {}",
                server_data.players_info.values(),
                server_name
            );
            server_data
        }
        None => {
            tracing::info!("no server data for server: {}", server_name);
            return;
        }
    };

    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if server_data
            .players_info
            .values()
            .any(|player_info| player_info.player_ids.contains(&(other_id as u32)))
        {
            let _ = sender.send(ServerEvent::UpdateServerData(Box::new(server_data.clone())));
        }
    }
}

#[cfg(feature = "server")]
fn update_clients_ongoing_games() {
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::UpdateOngoingGames(
            GAMES_MANAGER.lock().unwrap().ongoing_games.clone(),
        ));
    }
}

#[cfg(feature = "server")]
pub fn update_app_after_atk(server_name: &str, selected_atk_name: Option<&str>) {
    // get app by server name
    let gm = GAMES_MANAGER.lock().unwrap();
    let mut app = match gm.servers_data.get(server_name) {
        Some(server_data) => server_data.app.clone(),
        None => {
            tracing::error!("No application found for server name: {}", server_name);
            return;
        }
    };
    drop(gm);
    // launch attack
    let _ = app.game_manager.launch_attack(selected_atk_name);
    // update clients
    update_clients_app(server_name, &app);
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
pub async fn process_ennemy_atk(server_name: &str, tx: mpsc::UnboundedSender<ServerOwnEvent>) {
    if let Some(app) = get_app_by_server_name(server_name)
        && app.game_manager.is_round_auto()
    {
        let nb_in_a_row = app.game_manager.process_nb_bosses_atk_in_a_row();
        let server_name = server_name.to_string(); // if it was &str
        tokio::spawn(async move {
            let mut i = 0;
            while i < nb_in_a_row {
                sleep(std::time::Duration::from_millis(3000)).await;
                let _ = tx.send(ServerOwnEvent::AutoAtkIsDone(server_name.clone()));
                tracing::info!("process_ennemy_atk in a row : {}", nb_in_a_row);
                i += 1;
            }
        });
    }
}

#[cfg(feature = "server")]
pub fn get_app_by_server_name(server_name: &str) -> Option<Application> {
    // get app by server name
    let gm = GAMES_MANAGER.lock().unwrap();
    let app = match gm.servers_data.get(server_name) {
        Some(server_data) => server_data.app.clone(),
        None => {
            tracing::error!("No application found for server name: {}", server_name);
            drop(gm);
            return None;
        }
    };
    drop(gm);
    Some(app)
}

#[cfg(feature = "server")]
pub fn get_server_data_by_server_name(server_name: &str) -> Option<ServerData> {
    // get server-data by server name
    let gm = GAMES_MANAGER.lock().unwrap();
    let server_data = match gm.servers_data.get(server_name) {
        Some(server_data) => server_data.clone(),
        None => {
            tracing::error!("No server data found for server name: {}", server_name);
            drop(gm);
            return None;
        }
    };
    drop(gm);
    Some(server_data)
}

#[cfg(feature = "server")]
pub fn add_server_data_with_player(
    app: &Application,
    server_name: &str,
    id: u32,
    player_name: &str,
) {
    let mut gm: std::sync::MutexGuard<'_, GameStateManager> = GAMES_MANAGER.lock().unwrap();
    gm.add_server_data(server_name, app);
    gm.add_player_to_server(server_name, player_name, id);
    tracing::info!("servers data keys: {:?}", gm.servers_data.keys());
}

#[cfg(feature = "server")]
fn update_lobby_page_after_joining_game(server_name: &str, player_name: &str, client_id: u32) {
    // update lobby page for the player who joined the game
    let mut gm: std::sync::MutexGuard<'_, GameStateManager> = GAMES_MANAGER.lock().unwrap();
    gm.add_player_to_server(server_name, player_name, client_id);
    drop(gm);
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
fn add_character_on_server_data(server_name: &str, player_name: &str, character_name: &str) {
    let mut gm = GAMES_MANAGER.lock().unwrap();
    if let Some(server_data) = gm.servers_data.get_mut(server_name) {
        // remove character from all players in server data
        server_data
            .players_info
            .entry(player_name.to_string())
            .or_default()
            .character_names
            .clear();
        server_data
            .players_info
            .entry(player_name.to_string())
            .or_default()
            .character_names
            .push(character_name.to_string());
        // find character in pm and set it as active for all players in server data
        server_data.app.game_manager.pm.active_heroes.clear();
        server_data.players_info.values().for_each(|player_info| {
            player_info
                .character_names
                .iter()
                .for_each(|character_name| {
                    if let Some(character) = server_data
                        .app
                        .game_manager
                        .pm
                        .all_heroes
                        .iter()
                        .find(|h| h.name == *character_name)
                    {
                        server_data
                            .app
                            .game_manager
                            .pm
                            .active_heroes
                            .push(character.clone());
                    } else {
                        tracing::error!(
                            "Character {} not found in pm for server {}",
                            character_name,
                            server_name
                        );
                    }
                });
        });
        tracing::debug!(
            "Player {} added character {} to server data for server {}",
            player_name,
            character_name,
            server_name
        );
    } else {
        tracing::error!("Server data not found for server: {}", server_name);
    }
    drop(gm);
    // comment active heroes for all players in server data
    tracing::debug!(
        "active heroes for server {}: {:?}",
        server_name,
        get_app_by_server_name(server_name).map(|app| app
            .game_manager
            .pm
            .active_heroes
            .iter()
            .map(|h| h.name.clone())
            .collect::<Vec<String>>())
    );
    update_clients_server_data(server_name);
    update_clients_app(
        server_name,
        &get_app_by_server_name(server_name).unwrap_or_default(),
    );
}

#[cfg(feature = "server")]
use crate::application::init_application;
#[cfg(feature = "server")]
use crate::common::{SAVED_GAME_MANAGER, SAVED_GAME_MANAGER_REPLAY};
use crate::websocket_handler::game_state::GamePhase;
use crate::websocket_handler::game_state::OnGoingGame;
use crate::websocket_handler::game_state::ServerData;
#[cfg(feature = "server")]
use async_std::task::sleep;
use dioxus::fullstack::{CborEncoding, WebSocketOptions, Websocket};
use dioxus::logger::tracing;
use dioxus::prelude::*;
#[cfg(feature = "server")]
use lib_rpg::common::paths_const::GAMES_DIR;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;
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
static SERVER_MANAGER: Lazy<Arc<Mutex<GameStateManager>>> =
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
    RequestSavedGameList,
    RequestOnGoingGamesList,
    LoadGame(PathBuf, String), // `PathBuf`: game path, `String`: player name
    ReplayGame(String),        // `String`: server name
    DisconnectFromServerData(String, String), // `String`: server name, `String`: player name
    RequestTargetedCharacter(String, String, String), // `String`: launcher name, `String`: server name, `String`: atk name
    RequestSetOneTarget(String, String, String, String), // `String`: launcher name, `String`: server name, `String`: atk name, `String`: target name
    SaveGame(String),                                    // `String`: server name
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    NewClientOnExistingPlayer(String, u32), // welcome message, player id
    AssignPlayerId(u32),                    // player id
    UpdateApplication(Box<Application>),
    ReconnectAllSessions(String, i64), // username, sql-id
    UpdateServerData(Box<ServerData>), // server data
    UpdateOngoingGames(Vec<OnGoingGame>),
    AnswerSavedGameList(Vec<PathBuf>), // list of saved games paths
    EndOfServerData,                   // server name
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
                                start_new_game_by_player(&server_name, false).await;
                            }
                            Ok(ClientEvent::InitializeGame(server_name, player_name)) => {
                                tracing::info!("{} is initializing a new game", server_name);
                                init_new_game_by_player(&server_name, client_id, &player_name).await;
                            }
                            Ok(ClientEvent::AddCharacterOnServerData(server_name, player_name, character_name)) => {
                                tracing::info!("{} is adding character {} to server data", player_name, character_name);
                                add_character_on_server_data(&server_name, &player_name, &character_name);
                                update_clients_server_data(&server_name);
                            }
                            Ok(ClientEvent::LaunchAttack(server_name, selected_atk)) => {
                                tracing::info!("A new atk has been launched with atk {} for server {}", selected_atk, server_name);
                                update_app_after_atk(&server_name, Some(&selected_atk));
                                update_clients_server_data(&server_name);
                                // is ennemy turn ? 
                                process_ennemy_atk(&server_name, tx_server.clone()).await;
                            }
                            Ok(ClientEvent::JoinServerData(server_name, player_name)) => {
                                tracing::info!("Player {} with id {} is joining server data for server {}", player_name, client_id, server_name);
                                update_lobby_page_after_joining_game(&server_name, &player_name, client_id);
                            }
                            Ok(ClientEvent::RequestSavedGameList) => {
                                tracing::info!("Client {} requested saved game list", client_id);
                                update_saved_game_list_display().await;
                            }
                            Ok(ClientEvent::LoadGame(game_path, player_name)) => {
                                tracing::info!("Player {} with id {} is loading game: {}", player_name, client_id, game_path.to_string_lossy());
                                load_game_by_player(game_path, player_name, client_id, false, None).await;
                            }
                            Ok(ClientEvent::RequestOnGoingGamesList) => {
                                tracing::info!("Client {} requested ongoing games list", client_id);
                                update_ongoing_games_list_display(client_id).await;
                            }
                            Ok(ClientEvent::ReplayGame(server_name)) => {
                                tracing::info!("Client {} requested replay game", client_id);
                                process_replay_game(&server_name, client_id).await;
                            }
                            Ok(ClientEvent::DisconnectFromServerData(server_name, player_name)) => {
                                tracing::info!("Client {} requested disconnection from server-data {}", client_id, server_name);
                                send_disconnection_to_server_data(client_id, &server_name, &player_name).await;
                            }
                            Ok(ClientEvent::RequestTargetedCharacter(server_name, launcher_name, atk_name)) => {
                                tracing::info!("Client {} requested update target with target {} and atk {}", client_id, launcher_name, atk_name);
                                request_set_targeted_characters(&server_name, &launcher_name, &atk_name);
                            }
                            Ok(ClientEvent::RequestSetOneTarget(server_name, launcher_name, atk_name, target_name)) => {
                                tracing::info!("Client {} requested update target with target {} and atk {}", client_id, launcher_name, atk_name);
                                request_set_one_target(&server_name, &launcher_name, &atk_name, &target_name);
                            }
                            Ok(ClientEvent::SaveGame(server_name)) => {
                                tracing::info!("Client {} requested save game", client_id);
                                process_save_game(&server_name).await;
                            }
                            Err(_) => {
                                // ClientEvent::ConnectionClosed
                                tracing::info!("Client {} disconnected", client_id);
                                // TODO get server name from GAMES_MANAGER by player id
                                send_disconnection_to_server_manager(client_id).await;
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
    let mut gm = SERVER_MANAGER.lock().unwrap();
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
    let mut gm = SERVER_MANAGER.lock().unwrap();
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
pub async fn send_disconnection_to_server_manager(cur_player_id: u32) {
    let mut sm = SERVER_MANAGER.lock().unwrap();
    let username = sm
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
    sm.players.retain(|player_name, ids| {
        if player_name == &username {
            ids.retain(|&id| id != cur_player_id);
            !ids.is_empty()
        } else {
            true
        }
    });
    tracing::info!(
        "All connected players after disconnection: {:?}",
        sm.players
    );
    // remove from servers data
    if let Some(server_data) = sm.servers_data.get_mut(&username) {
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
pub async fn send_disconnection_to_server_data(
    client_id: u32,
    server_name: &str,
    player_name: &str,
) {
    let mut sm = SERVER_MANAGER.lock().unwrap();
    let Some(is_owner_disconnecting) = sm
        .servers_data
        .get(server_name)
        .map(|server_data| server_data.owner_player_name == player_name)
    else {
        tracing::info!(
            "Player {} with id {} is disconnecting from server data {}, but no server data found for that server",
            player_name,
            client_id,
            server_name
        );
        return;
    };
    if is_owner_disconnecting {
        // remove ongoing game if exists for the server name
        sm.ongoing_games
            .retain(|ongoing_game| ongoing_game.server_name != server_name);
        drop(sm);
        update_clients_ongoing_games();
    }

    // send end of game to clients before deleting the ids from the server data, so that the clients can know which game is ending based on the server data they have
    send_end_of_serverdata(server_name, client_id, is_owner_disconnecting);

    // remove from servers data
    let mut sm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        if player_name == server_data.owner_player_name {
            // if the owner player is disconnecting, we consider that the server data is not relevant anymore, and we remove it
            sm.servers_data.remove(server_name);
            tracing::info!(
                "Owner player {} is disconnecting, removing server data for server {}",
                player_name,
                server_name
            );
        } else {
            server_data.players_info.retain(|_player_name, pl| {
                pl.player_ids.retain(|&id| id != client_id);
                !pl.player_ids.is_empty()
            });
            tracing::info!(
                "Player {} with id {} is disconnecting from server data {}, remaining players in server data: {:?}",
                player_name,
                client_id,
                server_name,
                server_data.players_info
            );
        }
    } else {
        tracing::info!(
            "Player {} with id {} is disconnecting from server data {}, but no server data found for that server",
            player_name,
            client_id,
            server_name
        );
    }
    drop(sm);
}

#[cfg(feature = "server")]
pub async fn start_new_game_by_player(server_name: &str, is_replay: bool) {
    let app = {
        let mut sm = SERVER_MANAGER.lock().unwrap();

        let Some(server_data) = sm.servers_data.get_mut(server_name) else {
            tracing::error!("No server data found for server name: {}", server_name);
            return;
        };

        if !is_replay {
            server_data.app.game_manager.start_game();
        }

        server_data.app.game_phase = GamePhase::Running;
        tracing::info!("Game started for server: {}", server_name);

        // clone what you need *inside* the lock
        server_data.app.clone()
    }; // sm is guaranteed dropped here

    // async work happens after the lock is gone
    save_game_manager_state(&app, SAVED_GAME_MANAGER).await;
    save_game_manager_state(&app, SAVED_GAME_MANAGER_REPLAY).await;

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
            // save the game manager state
            save_game_manager_state(&app, SAVED_GAME_MANAGER).await;
            save_game_manager_state(&app, SAVED_GAME_MANAGER_REPLAY).await;
            // update ongoing servers data list
            let mut gm: std::sync::MutexGuard<'_, GameStateManager> =
                SERVER_MANAGER.lock().unwrap();
            // remove ongoing game if already exists for the server name
            gm.ongoing_games
                .retain(|ongoing_game| ongoing_game.server_name != server_name);
            // add ongoing game
            gm.ongoing_games.push(OnGoingGame {
                path: app.game_manager.game_paths.current_game_dir.clone(),
                server_name: server_name.to_string(),
            });
            drop(gm);
            // add server data
            app.game_phase = GamePhase::InitGame;
            add_server_data_with_player(&app, server_name, id, player_name);
            // update for the clients connected to that server
            update_clients_server_data(server_name);
            update_clients_app(server_name, &app);
            update_clients_ongoing_games();
        }
        Err(_) => tracing::error!("no app"),
    }
}

#[cfg(feature = "server")]
async fn save_game_manager_state(app: &Application, save_game_name: &str) {
    let path = app
        .game_manager
        .game_paths
        .current_game_dir
        .join(save_game_name);
    match application::create_dir(app.game_manager.game_paths.current_game_dir.clone()).await {
        Ok(()) => {}
        Err(e) => tracing::error!("Failed to create directory: {}", e),
    }
    match application::save(
        path,
        serde_json::to_string_pretty(&app.game_manager.clone()).unwrap(),
    )
    .await
    {
        Ok(()) => tracing::info!("Game manager state saved successfully {}", save_game_name),
        Err(e) => tracing::error!("Failed to save game manager state: {}", e),
    }
}

#[cfg(feature = "server")]
fn update_clients_app(server_name: &str, app: &Application) {
    let mut gm = SERVER_MANAGER.lock().unwrap();
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
    tracing::info!("Updating clients with ongoing games");
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::UpdateOngoingGames(
            SERVER_MANAGER.lock().unwrap().ongoing_games.clone(),
        ));
    }
}

#[cfg(feature = "server")]
fn send_end_of_serverdata(server_name: &str, client_id: u32, is_owner_disconnecting: bool) {
    // get server data
    let sm = SERVER_MANAGER.lock().unwrap();
    let server_data = match sm.servers_data.get(server_name) {
        Some(server_data) => server_data.clone(),
        None => {
            tracing::info!("no server data for server: {}", server_name);
            return;
        }
    };
    drop(sm);
    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if (!is_owner_disconnecting && client_id == other_id as u32)
            || (is_owner_disconnecting
                && server_data
                    .players_info
                    .values()
                    .any(|player_info| player_info.player_ids.contains(&(other_id as u32))))
        {
            let _ = sender.send(ServerEvent::EndOfServerData);
        }
    }
}

#[cfg(feature = "server")]
pub fn update_app_after_atk(server_name: &str, selected_atk_name: Option<&str>) {
    // get app by server name

    use lib_rpg::game_state::GameStatus;
    let gm = SERVER_MANAGER.lock().unwrap();
    let mut app = match gm.servers_data.get(server_name) {
        Some(server_data) => server_data.app.clone(),
        None => {
            tracing::error!("No application found for server name: {}", server_name);
            return;
        }
    };
    drop(gm);
    // launch attack
    // case several ennemy-auto-atk in a row and one atk ended the game, the next atk should not reach.
    // and the state of the game should not be updated anymore
    if app.game_manager.game_state.status == GameStatus::EndOfGame {
        tracing::info!(
            "Game is already ended for server: {}, skipping atk processing",
            server_name
        );
        return;
    }
    let _ = app.game_manager.launch_attack(selected_atk_name);
    tracing::info!(
        "Attack launched for server: {},  atk: {:?}",
        server_name,
        selected_atk_name
    );
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
    let gm = SERVER_MANAGER.lock().unwrap();
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
    let gm = SERVER_MANAGER.lock().unwrap();
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
    let mut gm: std::sync::MutexGuard<'_, GameStateManager> = SERVER_MANAGER.lock().unwrap();
    gm.add_server_data(server_name, app, player_name);
    gm.add_player_to_server(server_name, player_name, id);
    tracing::info!("servers data keys: {:?}", gm.servers_data.keys());
}

#[cfg(feature = "server")]
fn update_lobby_page_after_joining_game(server_name: &str, player_name: &str, client_id: u32) {
    // update lobby page for the player who joined the game
    let mut gm: std::sync::MutexGuard<'_, GameStateManager> = SERVER_MANAGER.lock().unwrap();
    gm.add_player_to_server(server_name, player_name, client_id);
    drop(gm);
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
fn add_character_on_server_data(server_name: &str, player_name: &str, character_name: &str) {
    let mut gm = SERVER_MANAGER.lock().unwrap();
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

#[cfg(feature = "server")]
async fn update_saved_game_list_display() {
    let games_list = match application::get_game_list(GAMES_DIR.to_path_buf()).await {
        Ok(games) => {
            for game in &games {
                tracing::info!("Game: {}", game.to_string_lossy());
            }
            games
        }
        Err(e) => {
            tracing::error!("Error fetching game list: {}", e);
            vec![]
        }
    };

    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::AnswerSavedGameList(games_list.clone()));
    }
}

/// load game-manager by player, then update app and server data for the clients connected to the server of that game
/// if is_replay is true, it means that the game-manager is loaded for a replay, so we set is_game_running to true to display the correct page on the client side
#[cfg(feature = "server")]
async fn load_game_by_player(
    game_path: PathBuf,
    player_name: String,
    client_id: u32,
    is_replay: bool,
    server_name_opt: Option<String>,
) {
    let server_name = server_name_opt.unwrap_or_else(|| player_name.clone());

    let gm = match application::get_gamemanager_by_game_dir(game_path.clone(), is_replay).await {
        Ok(gm) => gm,
        Err(e) => {
            tracing::error!(
                "Error loading game manager for player {}: {}",
                player_name,
                e
            );
            return;
        }
    };

    let app = Application {
        game_manager: gm,
        server_name: server_name.clone(),
        game_phase: if is_replay {
            GamePhase::Running
        } else {
            GamePhase::InitGame
        },
    };

    // persist state (no locks involved)
    save_game_manager_state(&app, SAVED_GAME_MANAGER).await;
    save_game_manager_state(&app, SAVED_GAME_MANAGER_REPLAY).await;

    // ---- update ongoing games (lock scope #1) ----
    {
        let mut sm = SERVER_MANAGER.lock().unwrap();

        sm.ongoing_games.retain(|g| g.server_name != server_name);

        sm.ongoing_games.push(OnGoingGame {
            path: app.game_manager.game_paths.current_game_dir.clone(),
            server_name: app.server_name.clone(),
        });
    } // lock released here

    if !is_replay {
        add_server_data_with_player(&app, &app.server_name, client_id, &player_name);

        update_clients_app(
            &app.server_name,
            &get_app_by_server_name(&app.server_name).unwrap_or_default(),
        );
        update_clients_server_data(&app.server_name);
    } else {
        tracing::info!("Starting replay for server: {}", server_name);

        // ---- update server data by app (lock scope #2) ----
        let server_exists = {
            let mut sm = SERVER_MANAGER.lock().unwrap();

            if let Some(server_data) = sm.servers_data.get_mut(&server_name) {
                server_data.app = app.clone();
                true
            } else {
                false
            }
        }; // lock released here

        if !server_exists {
            tracing::error!("No server data found for server name: {}", server_name);
            return;
        }

        start_new_game_by_player(&server_name, is_replay).await;
    }

    update_clients_ongoing_games();
}

#[cfg(feature = "server")]
async fn update_ongoing_games_list_display(client_id: u32) {
    let gm = SERVER_MANAGER.lock().unwrap();
    let ongoing_games = gm.ongoing_games.clone();
    drop(gm);
    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if other_id as u32 == client_id {
            let _ = sender.send(ServerEvent::UpdateOngoingGames(ongoing_games.clone()));
        }
    }
}

#[cfg(feature = "server")]
async fn process_replay_game(server_name: &str, client_id: u32) {
    // get server data by server name
    let server_data = match get_server_data_by_server_name(server_name) {
        Some(server_data) => server_data,
        None => {
            tracing::error!("No server data found for server name: {}", server_name);
            return;
        }
    };
    let cur_game_path = match get_app_by_server_name(server_name) {
        Some(app) => app.game_manager.game_paths.current_game_dir.clone(),
        None => {
            tracing::error!("No application found for server name: {}", server_name);
            return;
        }
    };
    // load game by player with is_replay = true
    // load game-manager from file
    load_game_by_player(
        cur_game_path,
        server_data.owner_player_name.clone(),
        client_id,
        true,
        Some(server_name.to_string()),
    )
    .await;
}

#[cfg(feature = "server")]
fn request_set_targeted_characters(server_name: &str, launcher_name: &str, atk_name: &str) {
    let mut gm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = gm.servers_data.get_mut(server_name) {
        server_data
            .app
            .game_manager
            .pm
            .set_targeted_characters(launcher_name, atk_name);
    } else {
        tracing::error!("No server data found for server name: {}", server_name);
    }
    drop(gm);
    update_clients_app(
        server_name,
        &get_app_by_server_name(server_name).unwrap_or_default(),
    );
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
fn request_set_one_target(
    server_name: &str,
    launcher_name: &str,
    atk_name: &str,
    target_name: &str,
) {
    let mut gm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = gm.servers_data.get_mut(server_name) {
        server_data
            .app
            .game_manager
            .pm
            .set_one_target(launcher_name, atk_name, target_name);
    } else {
        tracing::error!("No server data found for server name: {}", server_name);
    }
    drop(gm);
    update_clients_app(
        server_name,
        &get_app_by_server_name(server_name).unwrap_or_default(),
    );
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
async fn process_save_game(server_name: &str) {
    // get server data by server name
    let server_data = match get_server_data_by_server_name(server_name) {
        Some(server_data) => server_data,
        None => {
            tracing::error!("No server data found for server name: {}", server_name);
            return;
        }
    };
    let cur_game_path = server_data
        .app
        .game_manager
        .game_paths
        .current_game_dir
        .clone()
        .join("game_manager.json");
    match application::create_dir(
        server_data
            .app
            .game_manager
            .game_paths
            .current_game_dir
            .clone(),
    )
    .await
    {
        Ok(()) => {
            tracing::info!("Directory created or already existing successfully")
        }
        Err(e) => tracing::info!("Failed to create directory: {}", e),
    }
    match application::save(
        cur_game_path,
        serde_json::to_string_pretty(&server_data.app.game_manager).unwrap(),
    )
    .await
    {
        Ok(()) => {
            tracing::trace!("save");
        }
        Err(e) => tracing::trace!("{}", e),
    }
}

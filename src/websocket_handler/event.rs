#[cfg(feature = "server")]
use crate::common::DATA_MANAGER;
#[cfg(feature = "server")]
use crate::utils::server_file_utils;
#[cfg(feature = "server")]
use crate::websocket_handler::common_event::SERVER_MANAGER;
#[cfg(feature = "server")]
use async_std::task::sleep;
use dioxus::fullstack::{CborEncoding, WebSocketOptions, Websocket};
use dioxus::logger::tracing;
use dioxus::prelude::*;
use lib_rpg::character_mod::character::Character;
use lib_rpg::common::constants::core_game_data_const::{
    SAVED_CORE_GAME_DATA, SAVED_CORE_GAME_DATA_REPLAY,
};
#[cfg(feature = "server")]
use lib_rpg::common::constants::paths_const::GAMES_DIR;
use lib_rpg::common::log_data::LogData;
use lib_rpg::server::core_game_data::CoreGameData;
use lib_rpg::server::server_manager::OnGoingGame;
use lib_rpg::server::server_manager::ServerData;
#[cfg(feature = "server")]
use lib_rpg::server::server_manager::{GamePhase, ServerManager};
use lib_rpg::utils;
use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientEvent {
    LoginAllSessions(String, i64),  // `String`: username, `i64`: sql-id
    RequestLogOut(String),          // `String`: username
    InitializeGame(String, String), // `String`: server_name, `String`: player_name
    AddCharacterOnServerData(String, String, String), // `String`: server_name, `String`: player_name, `String`: character_name
    StartGame(String),                                // `String`: server_name
    LaunchAttack(String, String),                     // `String`: server_name, `String`: atk name
    AddPlayer(String),                                // `String`: username
    JoinServerData(String, String), // `String`: server_name, `String`: player_name
    RequestSavedGameList(String),   // `String`: player_name
    RequestOnGoingGamesList,
    LoadGame(PathBuf, String), // `PathBuf`: game path, `String`: player name
    ReplayGame(String),        // `String`: server name
    DisconnectFromServerData(String, String), // `String`: server name, `String`: player name
    RequestTargetedCharacter(String, String, String), // `String`: launcher name, `String`: server name, `String`: atk name
    RequestSetOneTarget(String, String, String, String), // `String`: launcher name, `String`: server name, `String`: atk name, `String`: target name
    SaveGame(String, String), // `String`: server name, `String`: player name
    AddLog(String, Vec<LogData>), // `String`: server name, `Vec<LogData>`: log info to add (ex: ["Player1 used Fireball on Player2 for 30 damage", "Player2 is now burning and will take 5 damage for 3 turns"])
    RequestToggleEquip(String, String, String), // `String`: equipment unique name, `String`: player name, `String`: server name
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    InitClient(u32, Vec<Character>),        // player id, characters list
    NewClientOnExistingPlayer(String, u32), // welcome message, player id
    ReconnectAllSessions(String, i64),      // username, sql-id
    UpdateServerData(Box<ServerData>),      // server data
    UpdateOngoingGames(Vec<OnGoingGame>),
    AnswerSavedGameList(Vec<PathBuf>), // list of saved games paths
    ResetClientFromServerData,         // server name
    LogOut,
    SetAtkAnimation(bool), // true to set atk animation, false to reset it
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerOwnEvent {
    AutoAtkIsDone(String),    // servername
    StopAtkAnimation(String), // servername
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
            // Data manager
    let dm = {
        let dm = DATA_MANAGER.lock().unwrap();
        dm.clone()
    };
    let character_list = dm
        .all_heroes
        .clone();

            let _ = socket.send(ServerEvent::InitClient(client_id, character_list)).await;
            let _ = socket.send(ServerEvent::NewClientOnExistingPlayer(format!("Welcome! (id={})", client_id), client_id)).await;

            // Main loop: handle incoming socket messages and outgoing queued messages
            loop {
                use crate::websocket_handler::event_inventory::request_toggle_equip;

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
                                if get_core_game_data_by_server_name(&server_name).is_some(){
                                    update_core_game_data_after_atk(&server_name, None, tx_server.clone()).await;
                                }
                            }
                            Some(ServerOwnEvent::StopAtkAnimation(server_name)) => {
                                update_clients_end_of_atk_animation(&server_name, false);
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
                            Ok(ClientEvent::RequestLogOut(user_name)) => {
                                tracing::info!("{} is logged out", user_name);
                                send_logout_to_server(user_name, client_id);
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
                            }
                            Ok(ClientEvent::LaunchAttack(server_name, selected_atk)) => {
                                tracing::info!("A new atk has been launched with atk {} for server {}", selected_atk, server_name);
                                update_core_game_data_after_atk(&server_name, Some(&selected_atk), tx_server.clone()).await;
                                // is ennemy turn ? 
                                process_ennemy_atk(&server_name, tx_server.clone()).await;
                            }
                            Ok(ClientEvent::JoinServerData(server_name, player_name)) => {
                                tracing::info!("Player {} with id {} is joining server data for server {}", player_name, client_id, server_name);
                                update_lobby_page_after_joining_game(&server_name, &player_name, client_id);
                            }
                            Ok(ClientEvent::RequestSavedGameList(player_name)) => {
                                tracing::info!("Client {} requested saved game list", client_id);
                                update_saved_game_list_display(&player_name).await;
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
                            Ok(ClientEvent::SaveGame(server_name, player_name)) => {
                                tracing::info!("Client {} requested save game by {}", client_id, player_name);
                                process_save_game(&server_name, &player_name).await;
                            }
                            Ok(ClientEvent::AddLog(server_name, logs)) => {
                                tracing::info!("Client {} requested to add logs, len: {}", client_id, logs.len());
                                add_log_to_app(&server_name, logs);
                                update_clients_server_data(&server_name);
                            }
                            Ok(ClientEvent::RequestToggleEquip(equipment_unique_name, player_name, server_name)) => {
                                tracing::info!("Client {} requested to toggle equip for equipment {} by player {} on server {}", client_id, equipment_unique_name, player_name, server_name);
                                request_toggle_equip(&equipment_unique_name, &player_name, &server_name).await;
                            }
                            Err(_) => {
                                // ClientEvent::ConnectionClosed
                                tracing::info!("Client {} disconnected", client_id);
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
    let mut sm: std::sync::MutexGuard<'_, ServerManager> = SERVER_MANAGER.lock().unwrap();
    sm.players.entry(name.clone()).or_default().push(id);
    tracing::info!("All connected players: {:?}", sm.players);
}

#[cfg(feature = "server")]
pub fn login_all_sessions(username: String, sql_id: i64) {
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::ReconnectAllSessions(username.clone(), sql_id));
    }
}

#[cfg(feature = "server")]
pub fn send_logout_to_server(user_name: String, client_id: u32) {
    let mut sm = SERVER_MANAGER.lock().unwrap();
    // remove player from ServerManager
    sm.players
        .retain(|player_name, _| player_name != &user_name);
    // remove from servers data
    if let Some(server_data) = sm.servers_data.get_mut(&user_name) {
        server_data
            .players_data
            .players_info
            .retain(|player_name, _| player_name != &user_name);
    }
    tracing::info!("All connected players after logout: {:?}", sm.players);

    // update logged out client
    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if other_id as u32 == client_id {
            let _ = sender.send(ServerEvent::LogOut);
            break;
        }
    }
}

#[cfg(feature = "server")]
pub async fn send_disconnection_to_server_manager(client_id: u32) {
    use crate::auth_manager::server_fn::update_connection_status;

    // ----------------------------------------
    //  Extract username + mutate players
    // ----------------------------------------
    let username = {
        let mut sm = SERVER_MANAGER.lock().unwrap();

        let username = sm
            .players
            .iter()
            .find_map(|(player_name, ids)| ids.contains(&client_id).then(|| player_name.clone()))
            .unwrap_or_else(|| "Unknown".to_string());

        tracing::info!("Player {} with id {} is disconnecting", username, client_id);

        sm.players.retain(|player_name, ids| {
            if player_name == &username {
                ids.retain(|&id| id != client_id);
                !ids.is_empty()
            } else {
                true
            }
        });

        tracing::info!("All connected players: {:?}", sm.players);

        username
    }; // LOCK DROPPED HERE

    // ----------------------------------------
    //  DB update (no locks held)
    // ----------------------------------------
    match update_connection_status(username.clone(), false).await {
        Ok(_) => tracing::info!("{} disconnected from db", username),
        Err(e) => tracing::error!("DB error: {:?}", e),
    };

    // ----------------------------------------
    //  Modify server data
    // ----------------------------------------
    let other_clients: Vec<u32> = {
        let mut sm = SERVER_MANAGER.lock().unwrap();

        let mut server_data = match sm.get_server_data_by_player_id(client_id) {
            Some(sd) => sd,
            None => {
                tracing::error!(
                    "Player {} with id {} disconnecting, no server data found",
                    username,
                    client_id
                );
                return;
            }
        };

        server_data
            .players_data
            .players_info
            .retain(|player_name, pl| {
                if player_name == &username {
                    pl.player_ids.retain(|&id| id != client_id);
                    !pl.player_ids.is_empty()
                } else {
                    true
                }
            });

        // Collect affected client IDs before unlocking
        let ids = server_data
            .players_data
            .players_info
            .values()
            .flat_map(|p| p.player_ids.iter().copied())
            .collect::<Vec<u32>>();

        sm.ongoing_games.retain(|g| g.server_name != username);

        ids
    }; // LOCK DROPPED HERE

    // ----------------------------------------
    //  Notify clients (separate lock)
    // ----------------------------------------
    {
        let clients = CLIENTS.lock().unwrap();

        for other_id in other_clients {
            if let Some(sender) = clients.get(&(other_id as usize)) {
                let _ = sender.send(ServerEvent::ResetClientFromServerData);
            }
        }
    } // LOCK DROPPED

    update_clients_ongoing_games();
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
        .map(|server_data| server_data.players_data.owner_player_name == player_name)
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
    } else {
        drop(sm);
    }
    // send end of game to clients before deleting the ids from the server data, so that the clients can know which game is ending based on the server data they have
    send_end_of_serverdata(server_name, client_id, is_owner_disconnecting);

    // remove from servers data
    let mut sm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        if player_name == server_data.players_data.owner_player_name {
            // if the owner player is disconnecting, we consider that the server data is not relevant anymore, and we remove it
            sm.servers_data.remove(server_name);
            tracing::info!(
                "Owner player {} is disconnecting, removing server data for server {}",
                player_name,
                server_name
            );
        } else {
            server_data
                .players_data
                .players_info
                .retain(|_player_name, pl| {
                    pl.player_ids.retain(|&id| id != client_id);
                    !pl.player_ids.is_empty()
                });
            tracing::info!(
                "Player {} with id {} is disconnecting from server data {}, remaining players in server data: {:?}",
                player_name,
                client_id,
                server_name,
                server_data.players_data.players_info
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

    // update all clients
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
pub async fn start_new_game_by_player(server_name: &str, is_replay: bool) {
    let (core_game_data, server_owner) = {
        let mut sm = SERVER_MANAGER.lock().unwrap();
        let Some(server_data) = sm.servers_data.get_mut(server_name) else {
            tracing::error!(
                "start_new_game_by_player: No server data found for server name: {}",
                server_name
            );
            return;
        };

        // start_game only for initialized game the first time
        // not for replay
        // not for loaded
        if !is_replay && server_data.core_game_data.game_phase == GamePhase::InitGame {
            server_data.core_game_data.game_manager.start_game();
        }

        server_data.core_game_data.game_phase = GamePhase::Running;
        tracing::info!("Game started for server: {}", server_name);

        (
            server_data.core_game_data.clone(),
            server_data.players_data.owner_player_name.clone(),
        )
    }; // sm is guaranteed dropped here

    // async work happens after the lock is gone
    save_core_game_data(&core_game_data, SAVED_CORE_GAME_DATA, &server_owner).await;
    save_core_game_data(&core_game_data, SAVED_CORE_GAME_DATA_REPLAY, &server_owner).await;

    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
pub async fn init_new_game_by_player(server_name: &str, id: u32, player_name: &str) {
    let dm = DATA_MANAGER.lock().unwrap();
    // init a new game
    let mut core_game_data = CoreGameData::new(&dm, server_name);
    drop(dm);
    tracing::info!("New core game data created for player: {}", server_name);
    // update ongoing servers data list
    let mut sm = SERVER_MANAGER.lock().unwrap();
    // remove ongoing game if already exists for the server name
    sm.ongoing_games
        .retain(|ongoing_game| ongoing_game.server_name != server_name);
    // add ongoing game
    sm.ongoing_games.push(OnGoingGame {
        path: core_game_data
            .game_manager
            .game_paths
            .output_current_game_dir
            .clone(),
        server_name: server_name.to_string(),
    });
    drop(sm);
    // add server data
    core_game_data.game_phase = GamePhase::InitGame;
    // add first player
    core_game_data.players_nb = 0;
    add_server_data_with_player(&core_game_data, server_name, id, player_name);
    // update for the clients connected to that server
    update_clients_server_data(server_name);
    update_clients_ongoing_games();
}

#[cfg(feature = "server")]
fn get_current_game_path(player_name: &str, current_game_dir: &str) -> PathBuf {
    use crate::common::SAVED_DATA;
    let saved_dir: PathBuf = SAVED_DATA.join(PathBuf::from(player_name));

    saved_dir.join(current_game_dir)
}

#[cfg(feature = "server")]
pub fn update_clients_server_data(server_name: &str) {
    let Some(server_data) = get_server_data_by_server_name(server_name) else {
        tracing::error!(
            "update_clients_server_data: No server data found for server name: {}",
            server_name
        );
        return;
    };
    send_server_event_to_clients(
        server_name,
        &ServerEvent::UpdateServerData(Box::new(server_data.clone())),
    );
}

#[cfg(feature = "server")]
pub fn update_clients_end_of_atk_animation(server_name: &str, is_animated: bool) {
    send_server_event_to_clients(server_name, &ServerEvent::SetAtkAnimation(is_animated));
}

#[cfg(feature = "server")]
fn send_server_event_to_clients(server_name: &str, server_event: &ServerEvent) {
    let Some(server_data) = get_server_data_by_server_name(server_name) else {
        tracing::error!(
            "update_clients_server_data: No server data found for server name: {}",
            server_name
        );
        return;
    };
    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if server_data
            .players_data
            .players_info
            .values()
            .any(|player_info| player_info.player_ids.contains(&(other_id as u32)))
        {
            let _ = sender.send(server_event.clone());
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
            tracing::info!(
                "send_end_of_serverdata: no server data for server: {}",
                server_name
            );
            return;
        }
    };
    drop(sm);
    let clients = CLIENTS.lock().unwrap();
    for (&other_id, sender) in clients.iter() {
        if (!is_owner_disconnecting && client_id == other_id as u32)
            || (is_owner_disconnecting
                && server_data
                    .players_data
                    .players_info
                    .values()
                    .any(|player_info| player_info.player_ids.contains(&(other_id as u32))))
        {
            let _ = sender.send(ServerEvent::ResetClientFromServerData);
        }
    }
}

#[cfg(feature = "server")]
pub async fn update_core_game_data_after_atk(
    server_name: &str,
    selected_atk_name: Option<&str>,
    tx: mpsc::UnboundedSender<ServerOwnEvent>,
) {
    use lib_rpg::server::game_state::GameStatus;
    let mut sm: std::sync::MutexGuard<'_, ServerManager> = SERVER_MANAGER.lock().unwrap();
    let logs: Vec<LogData>;
    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        // launch attack
        // case several ennemy-auto-atk in a row and one atk ended the game, the next atk should not reach.
        // and the state of the game should not be updated anymore
        if server_data.core_game_data.game_manager.game_state.status == GameStatus::EndOfGame {
            tracing::info!(
                "Game is already ended for server: {}, skipping atk processing",
                server_name
            );
            return;
        }
        let _ = server_data
            .core_game_data
            .game_manager
            .launch_attack(selected_atk_name);
        logs = server_data
            .core_game_data
            .game_manager
            .game_state
            .last_result_atk
            .logs_atk
            .clone();
        tracing::info!(
            "Attack has been launched for server: {},  atk: {:?}, logs.len: {}",
            server_name,
            selected_atk_name,
            logs.len()
        );
    } else {
        tracing::error!(
            "update_core_game_data_after_atk: No server data found for server name: {}",
            server_name
        );
        return;
    }

    drop(sm);

    // update clients
    update_clients_end_of_atk_animation(server_name, true);
    update_clients_server_data(server_name);
    // spawn
    let server_name = server_name.to_owned(); // if it was &str
    tokio::spawn(async move {
        let _ = tx.send(ServerOwnEvent::StopAtkAnimation(server_name.to_owned()));
    });
}

#[cfg(feature = "server")]
pub async fn process_ennemy_atk(server_name: &str, tx: mpsc::UnboundedSender<ServerOwnEvent>) {
    if let Some(app) = get_core_game_data_by_server_name(server_name)
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
pub fn get_core_game_data_by_server_name(server_name: &str) -> Option<CoreGameData> {
    // get app by server name
    let sm = SERVER_MANAGER.lock().unwrap();
    let app = match sm.servers_data.get(server_name) {
        Some(server_data) => server_data.core_game_data.clone(),
        None => {
            tracing::error!("No application found for server name: {}", server_name);
            drop(sm);
            return None;
        }
    };
    drop(sm);
    Some(app)
}

#[cfg(feature = "server")]
pub fn get_server_data_by_server_name(server_name: &str) -> Option<ServerData> {
    // get server-data by server name
    let sm = SERVER_MANAGER.lock().unwrap();
    let server_data = match sm.servers_data.get(server_name) {
        Some(server_data) => server_data.clone(),
        None => {
            drop(sm);
            return None;
        }
    };
    drop(sm);
    Some(server_data)
}

#[cfg(feature = "server")]
pub fn add_server_data_with_player(
    app: &CoreGameData,
    server_name: &str,
    id: u32,
    player_name: &str,
) {
    let mut sm = SERVER_MANAGER.lock().unwrap();
    sm.add_server_data(server_name, app, player_name);
    sm.add_player_to_server(server_name, player_name, id);
    tracing::info!("servers data keys: {:?}", sm.servers_data.keys());
}

#[cfg(feature = "server")]
fn update_lobby_page_after_joining_game(server_name: &str, player_name: &str, client_id: u32) {
    // update lobby page for the player who joined the game
    let mut sm = SERVER_MANAGER.lock().unwrap();
    sm.add_player_to_server(server_name, player_name, client_id);
    drop(sm);
    update_clients_server_data(server_name);
}

// Used when GamePhase::InitGame
#[cfg(feature = "server")]
fn add_character_on_server_data(server_name: &str, player_name: &str, character_name: &str) {
    let dm = DATA_MANAGER.lock().unwrap();
    let mut sm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        // remove characters from one player in server data
        server_data
            .players_data
            .players_info
            .entry(player_name.to_string())
            .or_default()
            .character_id_names
            .clear();
        // set id_name based on character_name
        let new_id_name = format!(
            "{}_#{}",
            character_name,
            1 + server_data
                .core_game_data
                .game_manager
                .pm
                .get_nb_of_active_heroes_by_name(character_name)
        );
        // update player info in server data
        server_data
            .players_data
            .players_info
            .entry(player_name.to_string())
            .or_default()
            .character_id_names
            .push(new_id_name.clone());
        // find character in pm and set it as active for all players in server data
        server_data
            .core_game_data
            .game_manager
            .pm
            .active_heroes
            .clear();
        server_data.core_game_data.heroes_chosen.clear();
        server_data
            .players_data
            .players_info
            .iter()
            .for_each(|player_info| {
                player_info
                    .1
                    .character_id_names
                    .iter()
                    .for_each(|character_id_name| {
                        let local_character_name = character_id_name
                            .split("_#")
                            .next()
                            .unwrap_or(character_id_name);
                        tracing::info!(
                            "Finding character {} in pm for server {}",
                            local_character_name,
                            server_name
                        );
                        if let Some(character) = dm
                            .all_heroes
                            .iter()
                            .find(|h| h.db_full_name == *local_character_name)
                        {
                            let mut character = character.clone();
                            character.id_name = character_id_name.clone();

                            // update suffix id name
                            server_data
                                .core_game_data
                                .game_manager
                                .pm
                                .active_heroes
                                .push(character.clone());
                            server_data
                                .core_game_data
                                .heroes_chosen
                                .insert(player_info.0.clone(), character.id_name.clone());
                        } else {
                            tracing::error!(
                                "Character {} not found in pm for server {}",
                                character_id_name,
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
    drop(sm);
    // comment active heroes for all players in server data
    tracing::debug!(
        "active heroes for server {}: {:?}",
        server_name,
        get_core_game_data_by_server_name(server_name).map(|app| app
            .game_manager
            .pm
            .active_heroes
            .iter()
            .map(|h| h.id_name.clone())
            .collect::<Vec<String>>())
    );
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
async fn update_saved_game_list_display(player_name: &str) {
    use crate::common::SAVED_DATA;

    let saved_dir = SAVED_DATA.join(player_name).join(GAMES_DIR.to_path_buf());
    let games_list = match server_file_utils::get_game_list(saved_dir).await {
        Ok(games) => games,
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

    // TODO game path should be init at initialization of the game and not here, and it should not be based on player name but on server name, to avoid issues when several players with different names are playing on the same server
    let load_path = if is_replay {
        get_current_game_path(&player_name, game_path.to_str().unwrap_or_default())
    } else {
        game_path
    };

    let mut app = match get_core_game_data_by_dir(load_path.clone(), is_replay).await {
        Ok(get_app) => get_app,
        Err(e) => {
            tracing::error!(
                "Error loading game manager for player {}: {}",
                player_name,
                e
            );
            return;
        }
    };

    app.game_phase = if is_replay {
        GamePhase::Running
    } else {
        GamePhase::Loading
    };

    // persist state (no locks involved)
    save_core_game_data(&app, SAVED_CORE_GAME_DATA, &player_name).await;
    save_core_game_data(&app, SAVED_CORE_GAME_DATA_REPLAY, &player_name).await;

    // ---- update ongoing games (lock scope #1) ----
    {
        let mut sm = SERVER_MANAGER.lock().unwrap();

        sm.ongoing_games.retain(|g| g.server_name != server_name);

        sm.ongoing_games.push(OnGoingGame {
            path: app.game_manager.game_paths.output_current_game_dir.clone(),
            server_name: app.server_name.clone(),
        });
    } // lock released here

    if !is_replay {
        add_server_data_with_player(&app, &app.server_name, client_id, &player_name);
        update_clients_server_data(&app.server_name);
    } else {
        tracing::info!("Starting replay for server: {}", server_name);

        // ---- update server data by app (lock scope #2) ----
        let server_exists = {
            let mut sm = SERVER_MANAGER.lock().unwrap();
            if let Some(server_data) = sm.servers_data.get_mut(&server_name) {
                server_data.core_game_data = app.clone();
                true
            } else {
                false
            }
        }; // lock released here

        if !server_exists {
            tracing::error!(
                "load_game_by_player: No server data found for server name: {}",
                server_name
            );
            return;
        }
    }

    update_clients_ongoing_games();
}

#[cfg(feature = "server")]
async fn update_ongoing_games_list_display(client_id: u32) {
    let sm = SERVER_MANAGER.lock().unwrap();
    let ongoing_games = sm.ongoing_games.clone();
    drop(sm);
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
            tracing::error!(
                "process_replay_game: No server data found for server name: {}",
                server_name
            );
            return;
        }
    };
    let cur_game_path = match get_core_game_data_by_server_name(server_name) {
        Some(app) => app.game_manager.game_paths.output_current_game_dir.clone(),
        None => {
            tracing::error!("No application found for server name: {}", server_name);
            return;
        }
    };
    // load game by player with is_replay = true
    // load game-manager from file
    load_game_by_player(
        cur_game_path,
        server_data.players_data.owner_player_name.clone(),
        client_id,
        true,
        Some(server_name.to_string()),
    )
    .await;
    start_new_game_by_player(server_name, true).await;
}

#[cfg(feature = "server")]
fn request_set_targeted_characters(server_name: &str, launcher_name: &str, atk_name: &str) {
    let mut sm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        server_data
            .core_game_data
            .game_manager
            .pm
            .set_targeted_characters(launcher_name, atk_name);
    } else {
        tracing::error!(
            "request_set_targeted_characters: No server data found for server name: {}",
            server_name
        );
    }
    drop(sm);
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
fn request_set_one_target(
    server_name: &str,
    launcher_name: &str,
    atk_name: &str,
    target_name: &str,
) {
    let mut sm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        server_data.core_game_data.game_manager.pm.set_one_target(
            launcher_name,
            atk_name,
            target_name,
        );
    } else {
        tracing::error!(
            "request_set_one_target: No server data found for server name: {}",
            server_name
        );
    }
    drop(sm);
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
async fn process_save_game(server_name: &str, player_name: &str) {
    // get server data by server name
    let server_data = match get_server_data_by_server_name(server_name) {
        Some(server_data) => server_data,
        None => {
            tracing::error!(
                "process_save_game: No server data found for server name: {}",
                server_name
            );
            return;
        }
    };
    save_core_game_data(
        &server_data.core_game_data,
        SAVED_CORE_GAME_DATA,
        player_name,
    )
    .await;
}

#[cfg(feature = "server")]
fn add_log_to_app(server_name: &str, logs: Vec<LogData>) {
    let mut sm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        server_data.core_game_data.game_manager.logs.extend(logs);
    }
    drop(sm);
}

#[cfg(feature = "server")]
pub async fn save_core_game_data(
    core_game_data: &CoreGameData,
    save_game_name: &str,
    player_name: &str,
) {
    // create dir
    use crate::common::SAVED_DATA;
    let saved_dir: PathBuf = SAVED_DATA.join(PathBuf::from(player_name));
    let saved_dir = saved_dir.join(
        core_game_data
            .game_manager
            .game_paths
            .output_current_game_dir
            .clone(),
    );
    match server_file_utils::create_dir(saved_dir.clone()).await {
        Ok(()) => {}
        Err(e) => tracing::error!("Failed to create directory: {}", e),
    }
    // save game
    let cur_game_path = saved_dir.join(save_game_name);
    match server_file_utils::save(
        cur_game_path,
        serde_json::to_string_pretty(&core_game_data.clone()).unwrap(),
    )
    .await
    {
        Ok(()) => tracing::info!("Core game data saved successfully {}", save_game_name),
        Err(e) => tracing::error!("Failed to save Core game data: {}", e),
    }
}

#[server]
pub async fn get_core_game_data_by_dir(
    game_dir_path: PathBuf,
    is_replay: bool,
) -> Result<CoreGameData, ServerFnError> {
    let core_game_data_file = if is_replay {
        game_dir_path.join(Path::new(SAVED_CORE_GAME_DATA_REPLAY))
    } else {
        game_dir_path.join(Path::new(SAVED_CORE_GAME_DATA))
    };
    if let Ok(value) = utils::read_from_json::<_, CoreGameData>(&core_game_data_file) {
        Ok(value)
    } else {
        Err(ServerFnError::new(format!(
            "Failed to read game state {:?}",
            game_dir_path
        )))
    }
}

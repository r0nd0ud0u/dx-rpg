use std::{collections::HashMap, path::PathBuf};

use crate::application::Application;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GameStateManager {
    /// key is player_name, value is a list of player_id (to handle multiple connections with the same player name, e.g. multiple tabs)
    pub players: HashMap<String, Vec<u32>>,
    /// List of paths to ongoing games, used to display on the load game page and to reconnect to ongoing games on server restart
    pub ongoing_games: Vec<OnGoingGame>,
    /// key is server_name, value is the server data (app state and players connected to the server)
    pub servers_data: HashMap<String, ServerData>,
    /// List of paths to saved games, used to display on the load game page
    pub saved_games_list: Vec<PathBuf>,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OnGoingGame {
    pub path: PathBuf,
    pub server_name: String,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ServerData {
    pub app: Application,
    pub players_info: HashMap<String, PlayerInfo>,
    pub owner_player_name: String,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PlayerInfo {
    pub character_names: Vec<String>,
    pub player_ids: Vec<u32>,
}

impl GameStateManager {
    pub fn new() -> Self {
        GameStateManager {
            players: HashMap::new(),
            ongoing_games: Vec::new(),
            servers_data: HashMap::new(),
            saved_games_list: Vec::new(),
        }
    }

    pub fn add_player(&mut self, player_name: String, player_id: u32) {
        self.players
            .entry(player_name.clone())
            .or_default()
            .push(player_id);
    }

    pub fn add_server_data(&mut self, server_name: &str, app: &Application, player_name: &str) {
        self.servers_data.insert(
            server_name.to_string(),
            ServerData {
                app: app.clone(),
                players_info: HashMap::new(),
                owner_player_name: player_name.to_string(),
            },
        );
    }

    pub fn add_player_to_server(&mut self, server_name: &str, player_name: &str, player_id: u32) {
        if let Some(server_data) = self.servers_data.get_mut(server_name) {
            server_data
                .players_info
                .entry(player_name.to_string())
                .or_default()
                .player_ids
                .push(player_id);
        }
    }

    /// Get the server data associated with a given player ID by searching through the servers data.
    pub fn get_server_data_by_player_id(&self, player_id: u32) -> Option<ServerData> {
        for server_data in self.servers_data.values() {
            for player_info in server_data.players_info.values() {
                if player_info.player_ids.contains(&player_id) {
                    return Some(server_data.clone());
                }
            }
        }
        None
    }
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum GamePhase {
    #[default]
    Default,
    InitGame,
    Running,
    Ended,
}

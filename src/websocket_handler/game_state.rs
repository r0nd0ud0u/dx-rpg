use std::{collections::HashMap, path::PathBuf};

use crate::application::Application;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GameStateManager {
    /// key is player_name, value is a list of player_id (to handle multiple connections with the same player name, e.g. multiple tabs)
    pub players: HashMap<String, Vec<u32>>,
    /// List of paths to ongoing games, used to display on the load game page and to reconnect to ongoing games on server restart
    pub ongoing_games_path: Vec<PathBuf>,
    /// key is server_name, value is the server data (app state and players connected to the server)
    pub servers_data: HashMap<String, ServerData>,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ServerData {
    pub app: Application,
    pub players: HashMap<String, PlayerInfo>,
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
            ongoing_games_path: Vec::new(),
            servers_data: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player_name: String, player_id: u32) {
        self.players
            .entry(player_name.clone())
            .or_default()
            .push(player_id);
    }

    pub fn add_server_data(&mut self, server_name: &str, app: &Application) {
        self.servers_data.insert(
            server_name.to_string(),
            ServerData {
                app: app.clone(),
                players: HashMap::new(),
            },
        );
    }

    pub fn add_player_to_server(&mut self, server_name: &str, player_name: &str, player_id: u32) {
        if let Some(server_data) = self.servers_data.get_mut(server_name) {
            server_data
                .players
                .entry(player_name.to_string())
                .or_default()
                .player_ids
                .push(player_id);
        }
    }
}

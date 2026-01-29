use std::{collections::HashMap, path::PathBuf};

use crate::application::Application;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GameStateManager {
    pub players: HashMap<String, Vec<u32>>, // key is player_name, value is player_id
    pub ongoing_games_path: Vec<PathBuf>,
    pub servers_data: HashMap<String, ServerData>, // key is server_name
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ServerData {
    pub app: Application,
    pub players: HashMap<String, Vec<u32>>, // key is player_name, value is player_id
}

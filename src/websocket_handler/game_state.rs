use std::{collections::HashMap, path::PathBuf};

use crate::application::Application;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GameStateManager {
    pub players: HashMap<u32, String>, // key is player_id, value is player_name
    pub ongoing_games_path: Vec<PathBuf>,
    pub servers_data: HashMap<String, ServerData>, // key is server_name
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ServerData {
    pub app: Application,
    pub clients_ids: Vec<u32>,
}

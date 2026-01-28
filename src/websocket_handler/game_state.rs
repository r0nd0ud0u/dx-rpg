use std::path::PathBuf;

use crate::application::Application;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GameStateWebsocket {
    pub players: Vec<String>,
    pub ongoing_games_path: Vec<PathBuf>,
    pub ongoing_games: HashMap<String, Application>,
}

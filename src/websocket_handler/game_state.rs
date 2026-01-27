use std::path::PathBuf;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GameStateWebsocket {
    pub players: Vec<String>,
    pub ongoing_games: Vec<PathBuf>,
}

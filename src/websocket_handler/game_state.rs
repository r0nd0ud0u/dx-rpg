#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameStateWebsocket {
    pub players: Vec<String>,
}

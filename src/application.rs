use dioxus::prelude::*;
use dioxus::{prelude::server, prelude::ServerFnError};
use lib_rpg::game_manager::{GameManager, GamePaths};
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Application {
    pub game_manager: GameManager,
}

#[server]
pub async fn try_new() -> Result<Application, ServerFnError> {
    match GameManager::try_new("offlines") {
        Ok(gm) => Ok(Application { game_manager: gm }),
        Err(_) => Err(ServerFnError::Request(
            "Failed to create GameManager".to_string(),
        )),
    }
}

#[server]
pub async fn save(path: String, value: String) -> Result<(), ServerFnError> {
    fs::write(path, value)?;
    Ok(())
}

#[server]
pub async fn start_game(paths: GamePaths) -> Result<(), ServerFnError> {
    if let Err(e) = fs::create_dir_all(paths.root) {
        eprintln!("Failed to create directory: {}", e);
    }
    if let Err(e) = fs::create_dir_all(paths.characters) {
        eprintln!("Failed to create directory: {}", e);
    }
    if let Err(e) = fs::create_dir_all(paths.game_state) {
        eprintln!("Failed to create directory: {}", e);
    }
    if let Err(e) = fs::create_dir_all(paths.loot) {
        eprintln!("Failed to create directory: {}", e);
    }
    Ok(())
}

#[server]
pub async fn sleep_from_millis(millis: u64) -> Result<(), ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(Duration::from_millis(millis)).await;
    Ok(())
}

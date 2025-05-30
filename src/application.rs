use dioxus::prelude::*;
use dioxus::{prelude::server, prelude::ServerFnError};
use lib_rpg::game_manager::{GameManager, GamePaths};
use lib_rpg::utils::{self, list_dirs_in_dir};
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Application {
    pub game_manager: GameManager,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OngoingGames {
    pub all_games: Vec<PathBuf>,
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
pub async fn create_dir(path: PathBuf) -> Result<(), ServerFnError> {
    if let Err(e) = fs::create_dir_all(path) {
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

#[server]
pub async fn get_game_list(game_dir_path: PathBuf) -> Result<Vec<PathBuf>, ServerFnError> {
    println!("Fetching game list from: {:?}", game_dir_path);
    let games_list = list_dirs_in_dir(&game_dir_path)?;
    println!("List games: {:?}", games_list);
    Ok(games_list)
}

#[server]
pub async fn log_debug(message: String) -> Result<(), ServerFnError> {
    println!("DEBUG: {}", message);
    Ok(())
}

#[server]
pub async fn get_gamemanager_by_game_dir(
    game_dir_path: PathBuf,
) -> Result<GameManager, ServerFnError> {
    let game_manager_file = game_dir_path.join(Path::new("game_manager.json"));
    if let Ok(value) = utils::read_from_json::<_, GameManager>(&game_manager_file) {
        Ok(value)
    } else {
        Err(ServerFnError::Request(
            "Failed to read game state".to_string(),
        ))
    }
}

#[server]
pub async fn delete_ongoing_game_status() -> Result<(), ServerFnError> {
    Ok(())
}

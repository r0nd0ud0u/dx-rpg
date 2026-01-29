use dioxus::prelude::*;
use dioxus::{prelude::ServerFnError, prelude::server};
use lib_rpg::game_manager::GameManager;
use lib_rpg::utils::{self, list_dirs_in_dir};
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Application {
    pub game_manager: GameManager,
    pub game_path: PathBuf,
    pub server_name: String,
    pub is_game_running: bool,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OngoingGames {
    pub all_games: Vec<PathBuf>,
}

#[server]
pub async fn try_new() -> Result<Application, ServerFnError> {
    match GameManager::try_new("offlines") {
        Ok(gm) => Ok(Application {
            game_manager: gm,
            game_path: PathBuf::from(""),
            server_name: "Default".to_owned(),
            is_game_running: false,
        }),
        Err(_) => Err(ServerFnError::new(
            "Failed to create GameManager".to_string(),
        )),
    }
}

#[server]
pub async fn save(path: String, value: String) -> Result<(), ServerFnError> {
    match fs::write(path, value) {
        Ok(_) => Ok(()),
        Err(_) => Err(ServerFnError::new("Failed to save file".to_string())),
    }
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
    let games_list = match list_dirs_in_dir(&game_dir_path) {
        Ok(list) => list,
        Err(_) => return Err(ServerFnError::new("Failed to list games".to_string())),
    };
    println!("List games: {:?}", games_list);
    Ok(games_list)
}

#[server]
pub async fn delete_game(game_path: PathBuf) -> Result<(), ServerFnError> {
    println!("Deleting game from: {:?}", game_path);
    match fs::remove_dir_all(&game_path) {
        Ok(_) => (),
        Err(_) => return Err(ServerFnError::new("Failed to delete game".to_owned())),
    };
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
        Err(ServerFnError::new(format!(
            "Failed to read game state {:?}",
            game_dir_path
        )))
    }
}

#[server]
pub async fn read_ongoinggames_from_json(path: String) -> Result<OngoingGames, ServerFnError> {
    if let Ok(value) = utils::read_from_json::<_, OngoingGames>(&path) {
        Ok(value)
    } else {
        Err(ServerFnError::new(format!("Unknown file: {:?}", path)))
    }
}

use dioxus::prelude::*;
use dioxus::{prelude::ServerFnError, prelude::server};
use lib_rpg::game_manager::GameManager;
use lib_rpg::utils::{self, list_dirs_in_dir};
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{SAVED_GAME_MANAGER, SAVED_GAME_MANAGER_REPLAY};

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Application {
    pub game_manager: GameManager,
    pub server_name: String,
    pub is_game_running: bool,
}

impl Application {
    #[server]
    pub async fn try_new() -> Result<Application, ServerFnError> {
        match GameManager::try_new("offlines", false) {
            Ok(gm) => Ok(Application {
                game_manager: gm,
                server_name: "Default".to_owned(),
                is_game_running: false,
            }),
            Err(_) => Err(ServerFnError::new(
                "Failed to create GameManager".to_string(),
            )),
        }
    }
}

#[cfg(feature = "server")]
pub fn init_application(name: &str, app: &mut Application) {
    app.game_manager.init_new_game();
    // name of the server
    // TODO set server name based on user name + random string
    app.server_name = name.to_string();
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
    is_replay: bool,
) -> Result<GameManager, ServerFnError> {
    let game_manager_file = if is_replay {
        game_dir_path.join(Path::new(SAVED_GAME_MANAGER_REPLAY))
    } else {
        game_dir_path.join(Path::new(SAVED_GAME_MANAGER))
    };
    if let Ok(value) = utils::read_from_json::<_, GameManager>(&game_manager_file) {
        Ok(value)
    } else {
        Err(ServerFnError::new(format!(
            "Failed to read game state {:?}",
            game_dir_path
        )))
    }
}

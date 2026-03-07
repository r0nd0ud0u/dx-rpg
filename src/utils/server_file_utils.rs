use std::{fs, path::PathBuf};

use dioxus::logger::tracing;
use dioxus::prelude::*;
use lib_rpg::utils::list_dirs_in_dir;

#[server]
pub async fn save(path: PathBuf, value: String) -> Result<(), ServerFnError> {
    match fs::write(path, value) {
        Ok(_) => Ok(()),
        Err(_) => Err(ServerFnError::new("Failed to save file".to_string())),
    }
}

#[server]
pub async fn create_dir(path: PathBuf) -> Result<(), ServerFnError> {
    match fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(_) => Err(ServerFnError::new("Failed to create directory".to_string())),
    }
}

#[server]
pub async fn get_game_list(game_dir_path: PathBuf) -> Result<Vec<PathBuf>, ServerFnError> {
    let games_list = match list_dirs_in_dir(&game_dir_path) {
        Ok(list) => list,
        Err(_) => {
            return Err(ServerFnError::new(format!(
                "Failed to list games from {:?}",
                game_dir_path
            )));
        }
    };
    tracing::info!("List games length: {}", games_list.len());
    tracing::trace!("List games: {:?}", games_list);
    Ok(games_list)
}

#[server]
pub async fn delete_game(game_path: PathBuf) -> Result<(), ServerFnError> {
    tracing::info!("Deleting game from: {:?}", game_path);
    match fs::remove_dir_all(&game_path) {
        Ok(_) => (),
        Err(_) => return Err(ServerFnError::new("Failed to delete game".to_owned())),
    };
    Ok(())
}

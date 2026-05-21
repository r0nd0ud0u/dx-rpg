use std::{fs, path::PathBuf};

use dioxus::logger::tracing;
use dioxus::prelude::*;
use lib_rpg::utils::list_dirs_in_dir;
use serde::{Deserialize, Serialize};

/// Lightweight metadata for a saved game slot, used by the slot-picker UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveSlotInfo {
    /// Full path to the saved game directory
    pub path: PathBuf,
    /// Human-readable game/slot name (directory name)
    pub name: String,
    /// Last-modified time as a formatted string (RFC 3339 / ISO 8601)
    pub last_saved: String,
    /// Current scenario name, empty when not available
    pub current_scenario: String,
    /// Current scenario level
    pub scenario_level: u64,
}

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

/// Returns the configured maximum number of save slots from the `MAX_SAVES` environment variable.
/// Defaults to 3 if the variable is absent or cannot be parsed.
#[server]
pub async fn get_max_saves() -> Result<usize, ServerFnError> {
    let max = std::env::var("MAX_SAVES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3);
    Ok(max)
}

/// Returns metadata for all saved game slots belonging to `player_name`.
/// At most `MAX_SAVES` slots are returned.
#[server]
pub async fn get_save_slots(player_name: String) -> Result<Vec<SaveSlotInfo>, ServerFnError> {
    use crate::common::SAVED_DATA;
    use lib_rpg::common::constants::{
        core_game_data_const::SAVED_CORE_GAME_DATA, paths_const::GAMES_DIR,
    };

    let max_saves = std::env::var("MAX_SAVES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3);

    let saved_dir = SAVED_DATA.join(&player_name).join(GAMES_DIR.to_path_buf());

    let dirs = list_dirs_in_dir(&saved_dir).unwrap_or_default();

    let mut slots: Vec<SaveSlotInfo> = dirs
        .into_iter()
        .take(max_saves)
        .map(|game_path| {
            let name = game_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Try to read last-modified time from the save file
            let save_file = game_path.join(SAVED_CORE_GAME_DATA);
            let last_saved = fs::metadata(&save_file)
                .and_then(|m| m.modified())
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Local> = t.into();
                    dt.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|_| "—".to_string());

            // Try to extract scenario info from save file
            let (current_scenario, scenario_level) = fs::read_to_string(&save_file)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .map(|v| {
                    let scenario = v
                        .pointer("/game_manager/current_scenario/name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    let level = v
                        .pointer("/game_manager/current_scenario/level")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(0);
                    (scenario, level)
                })
                .unwrap_or_default();

            SaveSlotInfo {
                path: game_path,
                name,
                last_saved,
                current_scenario,
                scenario_level,
            }
        })
        .collect();

    // Pad with empty slots up to max_saves
    while slots.len() < max_saves {
        slots.push(SaveSlotInfo {
            path: PathBuf::new(),
            name: String::new(),
            last_saved: String::new(),
            current_scenario: String::new(),
            scenario_level: 0,
        });
    }

    Ok(slots)
}

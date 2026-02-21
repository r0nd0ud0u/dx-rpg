use dioxus::logger::tracing;
use dioxus::prelude::*;
use dioxus::{prelude::ServerFnError, prelude::server};
use lib_rpg::game_manager::GameManager;
use lib_rpg::utils::{self, list_dirs_in_dir};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{SAVED_APP, SAVED_APP_REPLAY};
use crate::websocket_handler::game_state::GamePhase;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Application {
    pub game_manager: GameManager,
    /// TODO use in ServerData struct
    pub server_name: String,
    pub game_phase: GamePhase,
    /// reload info: players_nb
    pub players_nb: i64,
    /// reload info: key: username, value: character-name
    pub heroes_chosen: HashMap<String, String>,
}

impl Application {
    #[server]
    pub async fn try_new() -> Result<Application, ServerFnError> {
        match GameManager::try_new("offlines", false) {
            Ok(gm) => Ok(Application {
                game_manager: gm,
                server_name: "Default".to_owned(),
                game_phase: GamePhase::Default,
                players_nb: 0,
                heroes_chosen: HashMap::new(),
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
        Err(_) => Err(ServerFnError::new("Failed to save file".to_string())),
    }
}

#[server]
pub async fn get_game_list(game_dir_path: PathBuf) -> Result<Vec<PathBuf>, ServerFnError> {
    let games_list = match list_dirs_in_dir(&game_dir_path) {
        Ok(list) => list,
        Err(_) => return Err(ServerFnError::new("Failed to list games".to_string())),
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

#[server]
pub async fn get_application_by_game_dir(
    game_dir_path: PathBuf,
    is_replay: bool,
) -> Result<Application, ServerFnError> {
    let app_file = if is_replay {
        game_dir_path.join(Path::new(SAVED_APP_REPLAY))
    } else {
        game_dir_path.join(Path::new(SAVED_APP))
    };
    if let Ok(value) = utils::read_from_json::<_, Application>(&app_file) {
        Ok(value)
    } else {
        Err(ServerFnError::new(format!(
            "Failed to read game state {:?}",
            game_dir_path
        )))
    }
}

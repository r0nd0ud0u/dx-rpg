#[cfg(feature = "server")]
use crate::common::DATA_MANAGER;
use crate::common::{SAVED_CORE_GAME_DATA, SAVED_CORE_GAME_DATA_REPLAY};
#[cfg(feature = "server")]
use crate::utils::server_file_utils;
use crate::websocket_handler::server_manager::GamePhase;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use dioxus::{prelude::ServerFnError, prelude::server};
use lib_rpg::game_manager::{GameManager, LogAtk};
use lib_rpg::utils;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Game core state, stored on the server and sent to clients
/// Those data are necessary to run/load/replay a game
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CoreGameData {
    pub game_manager: GameManager,
    /// TODO use in ServerData struct
    pub server_name: String,
    pub game_phase: GamePhase,
    /// reload info: players_nb
    pub players_nb: i64,
    /// reload info: key: username, value: character-name
    pub heroes_chosen: HashMap<String, String>,
    /// logs of the game, to display in the log sheet
    pub logs: Vec<LogAtk>,
}

impl CoreGameData {
    #[cfg(feature = "server")]
    pub async fn new() -> CoreGameData {
        let dm = DATA_MANAGER.lock().unwrap();
        let mut gm = GameManager::new("offlines", dm.equipment_table.clone());
        // set bosses
        dm.all_bosses.iter().for_each(|boss| {
            let mut boss_to_push = boss.clone();
            boss_to_push.id_name = format!(
                "{}_#{}",
                boss_to_push.db_full_name,
                1 + gm
                    .pm
                    .get_nb_of_active_bosses_by_name(&boss_to_push.db_full_name)
            );
            gm.pm.active_bosses.push(boss_to_push);
        });
        gm.pm.active_bosses = dm.all_bosses.clone();

        CoreGameData {
            game_manager: gm,
            server_name: "Default".to_owned(),
            game_phase: GamePhase::Default,
            players_nb: 0,
            heroes_chosen: HashMap::new(),
            logs: Vec::new(),
        }
    }
}

#[cfg(feature = "server")]
pub fn init(name: &str, core_game_data: &mut CoreGameData) {
    core_game_data.game_manager.init_new_game();
    // name of the server
    // TODO set server name based on user name + random string
    core_game_data.server_name = name.to_string();
}

#[server]
pub async fn get_core_game_data_by_dir(
    game_dir_path: PathBuf,
    is_replay: bool,
) -> Result<CoreGameData, ServerFnError> {
    let core_game_data_file = if is_replay {
        game_dir_path.join(Path::new(SAVED_CORE_GAME_DATA_REPLAY))
    } else {
        game_dir_path.join(Path::new(SAVED_CORE_GAME_DATA))
    };
    if let Ok(value) = utils::read_from_json::<_, CoreGameData>(&core_game_data_file) {
        Ok(value)
    } else {
        Err(ServerFnError::new(format!(
            "Failed to read game state {:?}",
            game_dir_path
        )))
    }
}

#[cfg(feature = "server")]
pub async fn save_core_game_data(core_game_data: &CoreGameData, save_game_name: &str, player_name: &str) {
    // create dir
    use crate::common::SAVED_DATA;
    let saved_dir: PathBuf = SAVED_DATA.join(PathBuf::from(player_name));
    let saved_dir = saved_dir.join(core_game_data.game_manager.game_paths.current_game_dir.clone());
    match server_file_utils::create_dir(saved_dir.clone()).await {
        Ok(()) => {}
        Err(e) => tracing::error!("Failed to create directory: {}", e),
    }
    // save game
    let cur_game_path = saved_dir.join(save_game_name);
    match server_file_utils::save(
        cur_game_path,
        serde_json::to_string_pretty(&core_game_data.clone()).unwrap(),
    )
    .await
    {
        Ok(()) => tracing::info!("Core game data saved successfully {}", save_game_name),
        Err(e) => tracing::error!("Failed to save Core game data: {}", e),
    }
}
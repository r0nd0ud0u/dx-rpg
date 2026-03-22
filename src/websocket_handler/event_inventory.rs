#[cfg(feature = "server")]
use crate::{
    common::DATA_MANAGER,
    websocket_handler::{common_event::SERVER_MANAGER, event::update_clients_server_data},
};
use lib_rpg::server::server_manager::ServerManager;

#[cfg(feature = "server")]
pub async fn request_toggle_equip(
    equipment_unique_name: &str,
    player_name: &str,
    server_name: &str,
) {
    let dm = DATA_MANAGER.lock().unwrap();
    let all_equipments = &dm.equipment_table;
    let mut sm: std::sync::MutexGuard<'_, ServerManager> = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name)
        && let Some(character_id_name) = server_data
            .players_data
            .get_first_character_name(player_name)
        && let Some(character) = server_data
            .core_game_data
            .game_manager
            .pm
            .get_mut_active_hero_character(&character_id_name)
    {
        character.toggle_equipment(equipment_unique_name, all_equipments);
    }
    drop(sm);
    drop(dm);
    // update all clients
    update_clients_server_data(server_name);
}

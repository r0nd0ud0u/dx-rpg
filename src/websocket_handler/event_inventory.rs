#[cfg(feature = "server")]
use crate::{
    common::DATA_MANAGER,
    websocket_handler::{common_event::SERVER_MANAGER, event::update_clients_server_data},
};
use dioxus::logger::tracing;
use lib_rpg::server::server_manager::ServerManager;

#[cfg(feature = "server")]
pub async fn request_toggle_equip(
    equipment_unique_name: &str,
    character_id_name: &str,
    server_name: &str,
) {
    use lib_rpg::character_mod::character::Character;

    let dm = DATA_MANAGER.lock().unwrap();
    let all_equipments = &dm.equipment_table;
    let mut sm: std::sync::MutexGuard<'_, ServerManager> = SERVER_MANAGER.lock().unwrap();

    // Toggle the equipment and dismiss its "new" badge. Interacting with an item
    // clears the notification, so the player can do it by clicking the item
    // directly instead of having to switch equipment tabs.
    let apply = |character: &mut Character| {
        character.toggle_equipment(equipment_unique_name, all_equipments);
        for item in character.inventory.equipments.values_mut().flatten() {
            if item.unique_name == equipment_unique_name {
                item.is_new = false;
            }
        }
    };

    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        let pm = &mut server_data.core_game_data.game_manager.pm;
        if let Some(character) = pm.get_mut_active_hero_character(character_id_name) {
            apply(character);
        }
        // `current_player` is a shadow working copy of the hero whose turn it is;
        // an attack writes it back over `active_heroes` (modify_active_character).
        // If we only mutate `active_heroes`, the next attack reverts the equip and
        // the dismissed badge reappears, so keep the shadow copy in sync too.
        if pm.current_player.id_name == character_id_name {
            apply(&mut pm.current_player);
        }
    }
    drop(sm);
    drop(dm);
    // update all clients
    update_clients_server_data(server_name);
}

/// Mark all equipment in `category_key` as seen for `character_id_name` so the
/// "new item" badge is dismissed.  The category key must match the serialised
/// `EquipmentJsonKey` variant name (PascalCase, e.g. "Chest").
#[cfg(feature = "server")]
pub fn request_mark_equip_seen(category_key: &str, character_id_name: &str, server_name: &str) {
    use lib_rpg::character_mod::equipment::EquipmentJsonKey;
    use strum::IntoEnumIterator;

    // Resolve the category from its display/serialised name
    let Some(category) = EquipmentJsonKey::iter().find(|k| k.to_string() == category_key) else {
        tracing::warn!(
            "request_mark_equip_seen: unknown category '{}'",
            category_key
        );
        return;
    };

    let mut sm = SERVER_MANAGER.lock().unwrap();
    if let Some(server_data) = sm.servers_data.get_mut(server_name)
        && let Some(character) = server_data
            .core_game_data
            .game_manager
            .pm
            .get_mut_active_hero_character(character_id_name)
    {
        character.inventory.mark_equipment_category_seen(&category);
    }
    drop(sm);
    update_clients_server_data(server_name);
}

#[cfg(feature = "server")]
use crate::websocket_handler::{common_event::SERVER_MANAGER, event::update_clients_server_data};
#[cfg(feature = "server")]
use dioxus::logger::tracing;
#[cfg(feature = "server")]
use lib_rpg::server::server_manager::ServerManager;

/// Spend a skill point on `talent_id` for `character_id_name`. Validation (cost,
/// prerequisites, capstone exclusivity) happens in `Character::unlock_talent`;
/// failures are logged and simply don't change server state.
#[cfg(feature = "server")]
pub fn request_unlock_talent(server_name: &str, character_id_name: &str, talent_id: &str) {
    use lib_rpg::character_mod::character::Character;

    let mut sm: std::sync::MutexGuard<'_, ServerManager> = SERVER_MANAGER.lock().unwrap();

    let apply = |character: &mut Character, tree: &lib_rpg::character_mod::talent::TalentTree| {
        if let Err(e) = character.unlock_talent(talent_id, tree) {
            tracing::warn!(
                "Cannot unlock talent '{}' for {}: {}",
                talent_id,
                character.id_name,
                e
            );
        }
    };

    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        let pm = &mut server_data.core_game_data.game_manager.pm;
        let Some(character) = pm.get_active_hero_character(character_id_name) else {
            tracing::warn!(
                "request_unlock_talent: character '{}' not found on server {}",
                character_id_name,
                server_name
            );
            return;
        };
        let Some(tree) = pm.talent_trees.get(&character.db_full_name).cloned() else {
            tracing::warn!(
                "request_unlock_talent: no talent tree for hero '{}'",
                character.db_full_name
            );
            return;
        };

        if let Some(character) = pm.get_mut_active_hero_character(character_id_name) {
            apply(character, &tree);
        }
        // `current_player` is a shadow working copy of the hero whose turn it is;
        // an attack writes it back over `active_heroes` (modify_active_character).
        // Keep it in sync so a mid-turn unlock isn't reverted by the next attack.
        if pm.current_player.id_name == character_id_name {
            apply(&mut pm.current_player, &tree);
        }
    }
    drop(sm);
    update_clients_server_data(server_name);
}

/// Undo every unlocked talent for `character_id_name` and refund all spent points.
#[cfg(feature = "server")]
pub fn request_respec_talents(server_name: &str, character_id_name: &str) {
    use lib_rpg::character_mod::character::Character;

    let mut sm: std::sync::MutexGuard<'_, ServerManager> = SERVER_MANAGER.lock().unwrap();

    let apply = |character: &mut Character, tree: &lib_rpg::character_mod::talent::TalentTree| {
        character.respec_talents(tree);
    };

    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        let pm = &mut server_data.core_game_data.game_manager.pm;
        let Some(character) = pm.get_active_hero_character(character_id_name) else {
            tracing::warn!(
                "request_respec_talents: character '{}' not found on server {}",
                character_id_name,
                server_name
            );
            return;
        };
        let Some(tree) = pm.talent_trees.get(&character.db_full_name).cloned() else {
            tracing::warn!(
                "request_respec_talents: no talent tree for hero '{}'",
                character.db_full_name
            );
            return;
        };

        if let Some(character) = pm.get_mut_active_hero_character(character_id_name) {
            apply(character, &tree);
        }
        if pm.current_player.id_name == character_id_name {
            apply(&mut pm.current_player, &tree);
        }
    }
    drop(sm);
    update_clients_server_data(server_name);
}

/// Clear the talent notification badge for `character_id_name` — call when the
/// player opens the Talents tab, mirroring `request_mark_equip_seen`.
#[cfg(feature = "server")]
pub fn request_mark_talent_seen(server_name: &str, character_id_name: &str) {
    let mut sm: std::sync::MutexGuard<'_, ServerManager> = SERVER_MANAGER.lock().unwrap();

    if let Some(server_data) = sm.servers_data.get_mut(server_name) {
        let pm = &mut server_data.core_game_data.game_manager.pm;
        if let Some(character) = pm.get_mut_active_hero_character(character_id_name) {
            character.talents.mark_points_seen();
        }
        if pm.current_player.id_name == character_id_name {
            pm.current_player.talents.mark_points_seen();
        }
    }
    drop(sm);
    update_clients_server_data(server_name);
}

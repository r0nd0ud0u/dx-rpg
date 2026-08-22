//! Client-only: a local, in-process game engine for offline mode — no server, no
//! network, calls lib-rpg's game logic directly. `event.rs`'s server-side handlers turn
//! out to be almost entirely multiplayer broadcast plumbing (websocket client registry,
//! per-server client lists, ...) around a thin core of plain lib-rpg calls — solo offline
//! play doesn't need any of that plumbing, just the same lib-rpg calls made directly.
//!
//! This module is the foundational proof that embedded game data
//! (`embedded_data.rs`) can drive a real, playable game state entirely client-side.
//! Wiring this into the UI (swapping `socket.send(ClientEvent::...)` for direct calls
//! here, behind a shared abstraction) is a separate, later piece.
#![cfg(not(feature = "server"))]

use lib_rpg::server::{
    core_game_data::CoreGameData, data_manager::DataManager, server_manager::GamePhase,
};

use crate::common::OFFLINE_PATH;

/// Constructs a fresh single-player `CoreGameData` for `universe` (e.g. `"lotr"`), with
/// `hero_names` (matched against `Character.db_full_name`) selected as the active party
/// and the game already started — mirrors what `init_new_game_by_player` +
/// `add_character_on_server_data` + `start_new_game_by_player` do server-side for a real
/// game, minus the multiplayer/broadcast bookkeeping solo play doesn't need.
pub fn new_local_game(universe: &str, hero_names: &[&str]) -> anyhow::Result<CoreGameData> {
    crate::embedded_data::register();
    let dm = DataManager::try_new(OFFLINE_PATH)?;

    let scenarios: Vec<_> = dm
        .all_scenarios
        .iter()
        .filter(|s| s.universe == universe)
        .cloned()
        .collect();
    let mut core = CoreGameData::new_with_scenarios(&dm, "local", scenarios)?;
    core.is_single_player = true;
    core.universe = universe.to_owned();

    core.game_manager.pm.active_heroes = hero_names
        .iter()
        .filter_map(|name| dm.all_heroes.iter().find(|h| h.db_full_name == *name))
        .cloned()
        .collect();
    if let Some(first_hero) = core.game_manager.pm.active_heroes.first() {
        core.game_manager.pm.current_player = first_hero.clone();
    }

    core.game_phase = GamePhase::Running;
    core.game_manager.start_game();
    Ok(core)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_local_game_produces_a_playable_lotr_party() {
        let dm = DataManager::try_new(OFFLINE_PATH).unwrap_or_else(|_| {
            crate::embedded_data::register();
            DataManager::try_new(OFFLINE_PATH).unwrap()
        });
        let lotr_hero_names: Vec<&str> = dm
            .all_heroes
            .iter()
            .filter(|h| h.universe == "lotr")
            .map(|h| h.db_full_name.as_str())
            .collect();
        assert!(
            !lotr_hero_names.is_empty(),
            "expected lotr heroes to pick from"
        );

        let core = new_local_game("lotr", &lotr_hero_names[..1]).expect("new_local_game");
        assert_eq!(core.universe, "lotr");
        assert!(core.is_single_player);
        assert_eq!(core.game_phase, GamePhase::Running);
        assert_eq!(core.game_manager.pm.active_heroes.len(), 1);
        assert!(
            !core.game_manager.pm.active_bosses.is_empty(),
            "load_next_scenario should have populated active_bosses"
        );
        assert_eq!(
            core.game_manager.pm.current_player.db_full_name,
            lotr_hero_names[0]
        );
    }

    /// The actual end-to-end proof: a real attack, launched entirely client-side (no
    /// server, no network), produces a real combat effect against a real boss.
    #[test]
    fn local_game_combat_round_trip() {
        let dm = DataManager::try_new(OFFLINE_PATH).unwrap_or_else(|_| {
            crate::embedded_data::register();
            DataManager::try_new(OFFLINE_PATH).unwrap()
        });
        let hero_name = dm
            .all_heroes
            .iter()
            .find(|h| h.universe == "lotr")
            .expect("at least one lotr hero")
            .db_full_name
            .clone();

        let atk_names: Vec<String> = new_local_game("lotr", &[&hero_name])
            .expect("new_local_game")
            .game_manager
            .pm
            .current_player
            .attacks_list
            .keys()
            .cloned()
            .collect();
        assert!(
            !atk_names.is_empty(),
            "hero should have at least one attack"
        );

        // Not every attack necessarily produces an effect against a boss on a fresh,
        // minimal single-hero party (e.g. an ally-only buff has nothing to target with
        // just one hero in the party) — try each on a freshly constructed game (so cost/
        // cooldown/turn side effects from one attempt don't affect the next) until one
        // does, proving at least one real attack resolves correctly end to end.
        let mut found_a_landed_effect = false;
        let (mut boss_hp_before, mut boss_hp_after) = (0u64, 0u64);
        for atk_name in &atk_names {
            let mut core = new_local_game("lotr", &[&hero_name]).expect("new_local_game");
            let launcher_id_name = core.game_manager.pm.current_player.id_name.clone();

            boss_hp_before = core
                .game_manager
                .pm
                .active_bosses
                .iter()
                .map(|b| b.stats.all_stats[lib_rpg::common::constants::stats_const::HP].current)
                .sum();

            let result = core.game_manager.launch_attack(Some(atk_name));
            // Captured before the call: launch_attack may advance the turn (a new
            // current_player) as a side effect, so comparing against the post-call
            // current_player would be checking the wrong thing.
            assert_eq!(result.launcher_id_name, launcher_id_name);

            if !result.new_game_atk_effects.is_empty() {
                found_a_landed_effect = true;
                boss_hp_after = core
                    .game_manager
                    .pm
                    .active_bosses
                    .iter()
                    .map(|b| b.stats.all_stats[lib_rpg::common::constants::stats_const::HP].current)
                    .sum();
                break;
            }
        }
        assert!(
            found_a_landed_effect,
            "expected at least one of the hero's {} attacks to produce an effect",
            atk_names.len()
        );

        assert!(
            boss_hp_after <= boss_hp_before,
            "a landed attack should not increase total boss HP (before={boss_hp_before}, after={boss_hp_after})"
        );
    }
}

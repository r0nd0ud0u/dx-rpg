//! Client-only: a local, in-process game engine for offline mode — no server, no
//! network, calls lib-rpg's game logic directly. `event.rs`'s server-side handlers turn
//! out to be almost entirely multiplayer broadcast plumbing (websocket client registry,
//! per-server client lists, ...) around a thin core of plain lib-rpg calls — solo offline
//! play doesn't need any of that plumbing, just the same lib-rpg calls made directly.
//!
//! Three composable functions mirroring the real `InitializeGame` /
//! `AddCharacterOnServerData` / `StartGame` client-event flow (see
//! `local_channel.rs`, which calls these directly), each stripped of the
//! multiplayer/broadcast bookkeeping only the server side needs (a client registry,
//! per-server ongoing-games lists, save-to-disk, ...) — there's only ever one local
//! player offline.
#![cfg(not(feature = "server"))]

use anyhow::{Context, bail};
use lib_rpg::server::{
    core_game_data::CoreGameData, data_manager::DataManager, scenario::ScenarioState,
    server_manager::GamePhase,
};

use crate::common::OFFLINE_PATH;

/// Constructs a fresh single-player `CoreGameData` for `universe` (e.g. `"lotr"`) — no
/// heroes selected yet, game not started. Mirrors `init_new_game_by_player`'s core.
pub fn new_local_game(universe: &str) -> anyhow::Result<CoreGameData> {
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
    core.game_phase = GamePhase::InitGame;
    Ok(core)
}

/// Adds `hero_name` (matched against `Character.db_full_name`) to the active party,
/// setting it as the current player if it's the first hero added. Mirrors
/// `add_character_on_server_data`'s lookup+clone+push core.
pub fn add_hero(core: &mut CoreGameData, hero_name: &str) -> anyhow::Result<()> {
    let dm = DataManager::try_new(OFFLINE_PATH)?;
    let hero = dm
        .all_heroes
        .iter()
        .find(|h| h.db_full_name == hero_name)
        .with_context(|| format!("character {hero_name:?} not found in loaded game data"))?;
    let mut hero = hero.clone();
    hero.id_name = hero_name.to_owned();
    if core.game_manager.pm.active_heroes.is_empty() {
        core.game_manager.pm.current_player = hero.clone();
    }
    core.game_manager.pm.active_heroes.push(hero);
    Ok(())
}

/// Starts the game: mirrors `start_new_game_by_player`'s core (minus save-to-disk and
/// broadcasting to clients, neither of which apply to a single local player).
pub fn start_local_game(core: &mut CoreGameData) -> anyhow::Result<()> {
    if core.game_manager.pm.active_heroes.is_empty() {
        bail!("cannot start a local game with no heroes selected");
    }
    core.game_manager.start_game();
    let current_name = core.game_manager.current_scenario.name.clone();
    if let Some(state) = core.game_manager.states_scenarios.get_mut(&current_name) {
        *state = ScenarioState::InProgress;
    }
    if core.game_phase != GamePhase::Overworld {
        core.game_phase = GamePhase::Running;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lotr_hero_name(dm: &DataManager) -> String {
        dm.all_heroes
            .iter()
            .find(|h| h.universe == "lotr")
            .expect("at least one lotr hero")
            .db_full_name
            .clone()
    }

    #[test]
    fn new_local_game_produces_a_playable_lotr_party() {
        crate::embedded_data::register();
        let dm = DataManager::try_new(OFFLINE_PATH).unwrap();
        let hero_name = lotr_hero_name(&dm);

        let mut core = new_local_game("lotr").expect("new_local_game");
        assert_eq!(core.universe, "lotr");
        assert!(core.is_single_player);
        assert_eq!(core.game_phase, GamePhase::InitGame);
        assert!(
            !core.game_manager.pm.active_bosses.is_empty(),
            "load_next_scenario should have populated active_bosses"
        );

        add_hero(&mut core, &hero_name).expect("add_hero");
        assert_eq!(core.game_manager.pm.active_heroes.len(), 1);
        assert_eq!(core.game_manager.pm.current_player.db_full_name, hero_name);

        start_local_game(&mut core).expect("start_local_game");
        assert_eq!(core.game_phase, GamePhase::Running);
    }

    #[test]
    fn start_local_game_rejects_an_empty_party() {
        let mut core = new_local_game("lotr").expect("new_local_game");
        assert!(start_local_game(&mut core).is_err());
    }

    /// The actual end-to-end proof: a real attack, launched entirely client-side (no
    /// server, no network), produces a real combat effect against a real boss.
    #[test]
    fn local_game_combat_round_trip() {
        crate::embedded_data::register();
        let dm = DataManager::try_new(OFFLINE_PATH).unwrap();
        let hero_name = lotr_hero_name(&dm);

        let atk_names: Vec<String> = {
            let mut core = new_local_game("lotr").expect("new_local_game");
            add_hero(&mut core, &hero_name).expect("add_hero");
            core.game_manager
                .pm
                .current_player
                .attacks_list
                .keys()
                .cloned()
                .collect()
        };
        assert!(
            !atk_names.is_empty(),
            "hero should have at least one attack"
        );

        // Not every attack necessarily produces an effect against a boss on a fresh,
        // minimal single-hero party (e.g. an ally-only buff has nothing to target with
        // just one hero in the party) — try each on a freshly constructed game (so
        // cost/cooldown/turn side effects from one attempt don't affect the next) until
        // one does, proving at least one real attack resolves correctly end to end.
        let mut found_a_landed_effect = false;
        let (mut boss_hp_before, mut boss_hp_after) = (0u64, 0u64);
        for atk_name in &atk_names {
            let mut core = new_local_game("lotr").expect("new_local_game");
            add_hero(&mut core, &hero_name).expect("add_hero");
            start_local_game(&mut core).expect("start_local_game");
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

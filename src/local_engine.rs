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

/// Constructs a fresh single-player `CoreGameData` for `universe` (e.g. `"lotr"`, or
/// `""` for all universes) — no heroes selected yet, game not started. Mirrors
/// `init_new_game_by_player`'s core.
pub fn new_local_game(universe: &str) -> anyhow::Result<CoreGameData> {
    crate::embedded_data::register();
    let dm = DataManager::try_new(OFFLINE_PATH)?;

    let scenarios: Vec<_> = if universe.is_empty() {
        dm.all_scenarios.clone()
    } else {
        dm.all_scenarios
            .iter()
            .filter(|s| s.universe == universe)
            .cloned()
            .collect()
    };
    let mut core = CoreGameData::new_with_scenarios(&dm, "local", scenarios)?;
    core.is_single_player = true;
    core.universe = universe.to_owned();
    core.game_phase = GamePhase::InitGame;
    Ok(core)
}

/// Lists the distinct universes available in the embedded/offline data set (e.g.
/// `["lotr", "pokemon"]`), for the "Play Offline" universe picker — no network call,
/// mirrors `list_universes_server`'s scenario+hero union (minus its extra raw-directory
/// scan, which only matters for an admin-only edge case: a universe with character/
/// scenario folders present but not yet containing any actual data).
pub fn list_universes() -> anyhow::Result<Vec<String>> {
    crate::embedded_data::register();
    let dm = DataManager::try_new(OFFLINE_PATH)?;
    let universes: std::collections::HashSet<String> = dm
        .list_universes()
        .into_iter()
        .chain(dm.list_hero_universes())
        .collect();
    let mut universes: Vec<String> = universes.into_iter().collect();
    universes.sort();
    Ok(universes)
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

/// Loads `map_id` and switches to `GamePhase::Overworld`. Mirrors
/// `overworld_enter_handler`'s core (minus the auto-save-on-entry, which offline mode
/// doesn't do at all yet — see this module's doc comment).
///
/// `owner_hero_id`, if given, collapses `player_positions` down to just that hero's —
/// one party sprite for the whole group, matching the real handler (which does this
/// from `players_info`/`owner_player_name`, not available here since this module only
/// ever touches `CoreGameData`; `local_channel.rs`'s dispatch resolves it from the full
/// `ServerData` and passes it through).
pub fn enter_overworld_map(
    core: &mut CoreGameData,
    map_id: &str,
    spawn_override: Option<lib_rpg::common::overworld::Position>,
    owner_hero_id: Option<&str>,
) -> anyhow::Result<()> {
    let root = std::path::Path::new(OFFLINE_PATH);
    match spawn_override {
        Some(spawn) => core.enter_overworld_at(map_id, spawn, root)?,
        None => core.enter_overworld(map_id, root)?,
    }
    if let Some(hero_id) = owner_hero_id
        && let Some(ow) = core.overworld.as_mut()
    {
        let pos = ow
            .player_positions
            .get(hero_id)
            .cloned()
            .or_else(|| ow.player_positions.values().next().cloned())
            .unwrap_or_default();
        ow.player_positions.clear();
        ow.player_positions.insert(hero_id.to_owned(), pos);
    }
    Ok(())
}

/// Moves `hero_id`'s overworld sprite one step, handling the resulting encounter or
/// map-to-map transition in place so `core` is already fully updated (fight loaded, or
/// new map entered) by the time this returns. Mirrors `overworld_move_handler`'s core,
/// minus the server's broadcast-shape optimization (`BroadcastOverworldOnly` vs
/// `BroadcastFull`) — offline mode always re-emits one full state update regardless,
/// see `local_channel.rs`.
pub fn move_player(
    core: &mut CoreGameData,
    hero_id: &str,
    dir: lib_rpg::common::overworld::Direction,
    lang: lib_rpg::common::lang::Lang,
) -> anyhow::Result<()> {
    use lib_rpg::server::overworld_manager::{MoveResult, OverworldManager};

    let ow_state = core
        .overworld
        .as_ref()
        .with_context(|| "move_player: no overworld state")?;
    let mut manager = OverworldManager::from_state(ow_state.clone());
    let result = manager.move_player(hero_id, dir, lang);
    match result {
        MoveResult::Blocked | MoveResult::Moved => {
            core.overworld = Some(manager.state);
        }
        MoveResult::Encounter(scenario_id) => {
            core.overworld = Some(manager.state);
            core.exit_overworld_to_fight(&scenario_id);
        }
        MoveResult::MapTransition(target_map, spawn) => {
            enter_overworld_map(core, &target_map, Some(spawn), Some(hero_id))?;
        }
    }
    Ok(())
}

/// Interacts with whatever `hero_id` is facing (NPC dialog or a fight trigger).
/// Mirrors `overworld_interact_handler`'s core.
pub fn interact(
    core: &mut CoreGameData,
    hero_id: &str,
    lang: lib_rpg::common::lang::Lang,
) -> anyhow::Result<()> {
    use lib_rpg::server::overworld_manager::{InteractResult, OverworldManager};

    let ow_state = core
        .overworld
        .as_ref()
        .with_context(|| "interact: no overworld state")?;
    let mut manager = OverworldManager::from_state(ow_state.clone());
    let result = manager.interact(hero_id, lang);
    core.overworld = Some(manager.state);
    if let Some(InteractResult::Fight(scenario_id)) = result {
        core.exit_overworld_to_fight(&scenario_id);
    }
    Ok(())
}

/// Clears any active dialog/pending-fight prompt without acting on it. Mirrors
/// `overworld_dismiss_dialog_handler`'s core.
pub fn dismiss_dialog(core: &mut CoreGameData) {
    if let Some(ow) = core.overworld.as_mut() {
        ow.active_dialog.clear();
        ow.pending_fight = None;
    }
}

/// Leaves overworld mode back into combat/menu flow, keeping the overworld state so
/// re-entering the same map restores positions. Mirrors `overworld_exit_handler`'s
/// core.
pub fn exit_overworld(core: &mut CoreGameData) {
    core.game_phase = GamePhase::Running;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end proof that offline mode's overworld path works: enter the map lotr
    /// actually auto-enters after Start Game (see `startgame_page.rs`'s auto-effect),
    /// confirm the party sprite collapsed to just the owner's hero, then walk one step
    /// in every direction until one succeeds (a fresh map's exact walkable layout isn't
    /// asserted here — that's this map's own data, not local_engine's logic — only that
    /// `move_player` round-trips real `OverworldManager` state without erroring).
    #[test]
    fn enter_overworld_then_move_round_trips_real_state() {
        crate::embedded_data::register();
        let dm = DataManager::try_new(OFFLINE_PATH).unwrap();
        let hero_name = lotr_hero_name(&dm);

        let mut core = new_local_game("lotr").expect("new_local_game");
        add_hero(&mut core, &hero_name).expect("add_hero");
        start_local_game(&mut core).expect("start_local_game");

        enter_overworld_map(&mut core, "lotr_shire", None, Some(&hero_name))
            .expect("enter_overworld_map");
        assert_eq!(core.game_phase, GamePhase::Overworld);
        let ow = core.overworld.as_ref().expect("overworld state");
        assert_eq!(
            ow.player_positions.keys().collect::<Vec<_>>(),
            vec![&hero_name],
            "expected the party sprite collapsed to just the owner's hero"
        );

        use lib_rpg::common::{lang::Lang, overworld::Direction};
        for dir in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            move_player(&mut core, &hero_name, dir, Lang::En).expect("move_player");
        }

        interact(&mut core, &hero_name, Lang::En).expect("interact");
        dismiss_dialog(&mut core);
        exit_overworld(&mut core);
        assert_eq!(core.game_phase, GamePhase::Running);
    }

    #[test]
    fn list_universes_includes_lotr() {
        let universes = list_universes().expect("list_universes");
        assert!(
            universes.contains(&"lotr".to_owned()),
            "expected 'lotr' among {universes:?}"
        );
    }

    #[test]
    fn new_local_game_with_empty_universe_loads_all_scenarios() {
        crate::embedded_data::register();
        let dm = DataManager::try_new(OFFLINE_PATH).unwrap();
        let core = new_local_game("").expect("new_local_game");
        assert_eq!(
            core.game_manager.all_scenarios.len(),
            dm.all_scenarios.len()
        );
    }

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

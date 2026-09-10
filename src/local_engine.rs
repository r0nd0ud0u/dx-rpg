//! Client-only: an in-process game engine for offline mode — no server, no network,
//! calling lib-rpg directly.
//!
//! `event.rs`'s server handlers are mostly multiplayer broadcast plumbing (client
//! registry, per-server lists, save-to-disk) around a thin core of lib-rpg calls. These
//! functions mirror the `InitializeGame` / `AddCharacterOnServerData` / `StartGame` flow
//! with that plumbing stripped — there is only ever one local player.
#![cfg(not(feature = "server"))]

use anyhow::{Context, bail};
use lib_rpg::{
    common::log_data::LogData,
    server::{
        core_game_data::CoreGameData, data_manager::DataManager, game_state::ConsumableUseResult,
        server_manager::GamePhase,
    },
    utils::format_string_with_timestamp,
};

use crate::common::OFFLINE_PATH;

/// Constructs a fresh single-player `CoreGameData` for `universe` (e.g. `"lotr"`, or
/// `""` for all universes) — no heroes selected yet, game not started. Mirrors
/// `init_new_game_by_player`'s core.
/// Scenarios belonging to `universe`, or all of them when it's empty — the offline
/// counterpart of the filter `set_universe_on_server_data` applies server-side.
fn filter_scenarios(dm: &DataManager, universe: &str) -> Vec<lib_rpg::server::scenario::Scenario> {
    if universe.is_empty() {
        dm.all_scenarios.clone()
    } else {
        dm.all_scenarios
            .iter()
            .filter(|s| s.universe == universe)
            .cloned()
            .collect()
    }
}

/// Same filter as [`filter_scenarios`], loading the embedded data set itself. Used by the
/// offline channel's `SetUniverse` handler, which changes the universe on an already-built
/// game rather than creating a new one.
pub fn scenarios_for_universe(
    universe: &str,
) -> anyhow::Result<Vec<lib_rpg::server::scenario::Scenario>> {
    crate::embedded_data::register();
    let dm = DataManager::try_new(OFFLINE_PATH)?;
    Ok(filter_scenarios(&dm, universe))
}

pub fn new_local_game(universe: &str) -> anyhow::Result<CoreGameData> {
    crate::embedded_data::register();
    let dm = DataManager::try_new(OFFLINE_PATH)?;

    let scenarios = filter_scenarios(&dm, universe);
    let mut core = CoreGameData::new_with_scenarios(&dm, "local", scenarios)?;
    core.is_single_player = true;
    core.universe = universe.to_owned();
    core.game_phase = GamePhase::InitGame;
    Ok(core)
}

/// Universes in the embedded data set, for the "Play Offline" picker. Mirrors
/// `list_universes_server`'s scenario+hero union, minus its raw-directory scan (an
/// admin-only case: folders present but still empty).
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
    core.game_manager.mark_current_scenario_in_progress();
    if core.game_phase != GamePhase::Overworld {
        core.game_phase = GamePhase::Running;
    }
    Ok(())
}

/// Loads `map_id` and switches to `GamePhase::Overworld`. Mirrors
/// `overworld_enter_handler`, minus auto-save-on-entry (not done offline yet).
///
/// `owner_hero_id` collapses `player_positions` to one party sprite. The real handler
/// reads it from `players_info`; this module only sees `CoreGameData`, so
/// `local_channel.rs` resolves it and passes it in.
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

/// Moves `hero_id` one step, resolving any encounter or map transition in place so `core`
/// is fully updated on return. Mirrors `overworld_move_handler`, minus the server's
/// broadcast-shape choice — offline always re-emits full state.
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

/// Flags the characters `consumable_name` may be used on, so `character_page.rs`
/// can highlight them. Mirrors `request_target_for_consumable_handler`'s core.
pub fn set_consumable_targets(core: &mut CoreGameData, consumable_name: &str, is_party: bool) {
    let pm = &mut core.game_manager.pm;
    let launcher_id = pm.current_player.id_name.clone();
    let consumable = if is_party {
        pm.party_consumables
            .iter()
            .find(|c| c.name == consumable_name)
            .cloned()
    } else {
        pm.current_player
            .inventory
            .consumables
            .iter()
            .find(|c| c.name == consumable_name)
            .cloned()
    };
    match consumable {
        Some(c) => pm.set_targeted_characters_for_consumable(&launcher_id, &c),
        None => dioxus::logger::tracing::warn!(
            "offline mode: consumable {consumable_name:?} not found on the current player"
        ),
    }
}

/// Uses a consumable in combat, from the player's inventory or the party stock. Mirrors
/// `use_potion_handler`/`use_party_potion_handler`.
///
/// Bumps `game_state.last_consumable_use.seq` — the only thing telling the client a potion
/// was drunk, and all `Navbar` watches to play the sound.
pub fn use_consumable_in_combat(
    core: &mut CoreGameData,
    consumable_name: &str,
    target_id_name: &str,
    is_party: bool,
) -> anyhow::Result<()> {
    let game_state = core.game_manager.game_state.clone();
    let pm = &mut core.game_manager.pm;
    let launcher_id = pm.current_player.id_name.clone();

    let effects = if is_party {
        pm.use_party_consumable_on_target(consumable_name, target_id_name, &game_state)
    } else {
        pm.use_consumable_on_target(consumable_name, target_id_name, &game_state)
    }
    .with_context(|| format!("using consumable {consumable_name:?} on {target_id_name:?}"))?;

    // The potion left the current player's copy of the inventory; push that back
    // into `active_heroes` or it reappears on the next update.
    pm.modify_active_character(&launcher_id);

    let party = if is_party { " (party)" } else { "" };
    let stat_delta: i64 = effects.iter().map(|e| e.real_amount_tx).sum();
    let header = format!("💊 {launcher_id} uses {consumable_name}{party} on {target_id_name}");
    let message = if stat_delta == 0 {
        header.clone()
    } else {
        format!("{header} ({stat_delta:+})")
    };
    core.game_manager.logs.push(LogData {
        message: format_string_with_timestamp(&message),
        color: String::new(),
    });
    core.last_action_header = header;
    core.game_manager.game_state.last_consumable_use = ConsumableUseResult {
        launcher_id_name: launcher_id,
        target_id_name: target_id_name.to_owned(),
        consumable_name: consumable_name.to_owned(),
        seq: game_state.last_consumable_use.seq + 1,
    };
    Ok(())
}

/// Uses a consumable outside combat: always self-administered by `hero_id_name`,
/// and unlike [`use_consumable_in_combat`] it costs no turn and draws no counter-
/// attack. Mirrors `use_overworld_consumable_handler`'s core.
pub fn use_overworld_consumable(
    core: &mut CoreGameData,
    hero_id_name: &str,
    consumable_name: &str,
    is_party: bool,
) -> anyhow::Result<()> {
    let game_state = core.game_manager.game_state.clone();
    let pm = &mut core.game_manager.pm;

    if is_party {
        pm.use_party_consumable(hero_id_name, consumable_name, &game_state)?;
    } else {
        let hero = pm
            .get_mut_active_hero_character(hero_id_name)
            .with_context(|| format!("hero {hero_id_name:?} is not in the active party"))?;
        let consumable = hero
            .inventory
            .consumables
            .iter()
            .find(|c| c.name == consumable_name)
            .cloned()
            .with_context(|| format!("{hero_id_name:?} has no {consumable_name:?}"))?;
        let launcher_stats = hero.stats.clone();
        hero.use_consumable(consumable, &game_state, &launcher_stats)?;
    }

    core.game_manager.logs.push(LogData {
        message: format_string_with_timestamp(&format!("💊 {hero_id_name} uses {consumable_name}")),
        color: String::new(),
    });
    core.game_manager.game_state.last_consumable_use = ConsumableUseResult {
        launcher_id_name: hero_id_name.to_owned(),
        target_id_name: hero_id_name.to_owned(),
        consumable_name: consumable_name.to_owned(),
        seq: game_state.last_consumable_use.seq + 1,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Enters the map lotr auto-enters after Start Game, checks the party sprite collapsed
    /// to the owner's hero, then walks in each direction until one step succeeds. The map's
    /// walkable layout is its own data — this only proves `move_player` round-trips.
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

        // An ally-only buff has nothing to target in a one-hero party, so try each attack
        // on a fresh game (no cost/cooldown carry-over) until one resolves.
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

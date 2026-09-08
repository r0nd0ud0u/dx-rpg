//! Client-only: the offline-mode backend for `GameChannel` (see `game_channel.rs`).
//! Turns a `ClientEvent` into real game-state changes by calling `local_engine`/lib-rpg
//! directly — no network, no websocket — and reports back via the exact same
//! `ServerEvent` shapes the UI already knows how to handle.
//!
//! Deliberately simple rather than matching the server's per-action optimized update
//! shapes (`UpdateCombat`/`UpdateOverworld`/...): every handled action just re-emits one
//! full `ServerEvent::UpdateServerData(Box::new(current_state))`, which the existing
//! receive loop in `main.rs`'s `App()` already applies correctly. There's exactly one
//! local player, so the bandwidth/broadcast-efficiency reasons those lighter update
//! shapes exist for multiplayer don't apply here.
#![cfg(not(feature = "server"))]

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use futures::{StreamExt, channel::mpsc, lock::Mutex as AsyncMutex};
use lib_rpg::server::{
    game_state::GameStatus, scenario::ScenarioState, server_manager::ServerData,
};

use crate::websocket_handler::event::{ClientEvent, ServerEvent};

// `pub(crate)`: the "Play Offline" entry point (login_page.rs) needs this exact value
// too — it must set `local_login_name_session` and `SERVER_NAME` (via
// `send_initialize_game`'s `user_name` argument) to the same string this module uses
// for `owner_player_name`, or the several `owner_player_name == local_login_name_session()`
// host-only-controls checks scattered through startgame_page.rs (load next scenario,
// save game, ...) would never pass during an offline session.
pub(crate) const LOCAL_PLAYER_NAME: &str = "Player";
const LOCAL_CLIENT_ID: u32 = 0;

/// Pause before each enemy attack of an auto round, matching the 3s the server paces
/// its `AutoAtkIsDone` events by (`event.rs`'s `process_ennemy_atk`) — long enough to
/// read the log line and hear the hit before the next boss swings.
const AUTO_ATK_DELAY: Duration = Duration::from_millis(3000);

#[derive(Clone)]
pub struct LocalChannel {
    tx: mpsc::UnboundedSender<ServerEvent>,
    // An async-aware lock, not `RefCell`: `recv` needs to hold this across an `.await`
    // (for the whole duration of `.next().await`), which a plain `RefCell`'s guard
    // isn't safe to do (it can't yield to other tasks, so a would-be second borrow
    // panics instead of just waiting its turn).
    rx: Rc<AsyncMutex<mpsc::UnboundedReceiver<ServerEvent>>>,
    state: Rc<RefCell<ServerData>>,
    /// Enemy attacks still owed for the current auto round — see [`Self::recv`].
    pending_auto_atks: Rc<Cell<i64>>,
}

impl LocalChannel {
    /// Constructs an inert local channel — cheap, safe to always create alongside the
    /// real websocket (see `GameChannel`), whether or not the user ever picks offline
    /// mode. Doesn't touch game data until `activate()` is called.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self {
            tx,
            rx: Rc::new(AsyncMutex::new(rx)),
            state: Rc::new(RefCell::new(ServerData::default())),
            pending_auto_atks: Rc::new(Cell::new(0)),
        }
    }

    /// Starts an offline single-player session: registers embedded game data, resets
    /// local state, and pushes the same `InitClient` a real server sends right after a
    /// websocket connects — so the character-select UI (which reads the hero list from
    /// that event) works unchanged. Call once, when the user picks "Play Offline".
    pub fn activate(&self) {
        crate::embedded_data::register();
        *self.state.borrow_mut() = ServerData::default();
        self.pending_auto_atks.set(0);

        let all_heroes =
            lib_rpg::server::data_manager::DataManager::try_new(crate::common::OFFLINE_PATH)
                .map(|dm| dm.all_heroes)
                .unwrap_or_else(|e| {
                    dioxus::logger::tracing::error!("offline mode: failed to load game data: {e}");
                    Vec::new()
                });
        self.push(ServerEvent::InitClient(LOCAL_CLIENT_ID, all_heroes));
    }

    /// Applies `msg` to the local game state and reports the result back, matching the
    /// shape `GameChannel::send` expects.
    pub fn send(&self, msg: ClientEvent) {
        // The server runs the enemy's turn right after the player's action, not as part
        // of it (`event.rs`: `update_core_game_data_after_atk` then `process_ennemy_atk`)
        // — mirror that split here, so `recv` can pace the enemy attacks out one by one.
        let player_acted = matches!(msg, ClientEvent::LaunchAttack(..));
        for event in dispatch(&self.state, msg) {
            self.push(event);
        }
        if player_acted {
            self.queue_auto_atks();
        }
    }

    /// The offline counterpart of the server's `process_ennemy_atk`: once the player has
    /// acted, count the bosses that act before the next hero gets a turn. Without this,
    /// offline combat simply stopped at the end of the player's attack — the enemy's turn
    /// was never played at all, so the boss never hit back.
    fn queue_auto_atks(&self) {
        let game_manager = &self.state.borrow().core_game_data.game_manager;
        if game_manager.is_round_auto() {
            self.pending_auto_atks
                .set(game_manager.process_nb_bosses_atk_in_a_row());
        }
    }

    /// Plays one owed enemy attack. `launch_attack(None)` is lib-rpg's "the character
    /// whose turn it is picks its own attack", the very call the server makes for each
    /// `AutoAtkIsDone`.
    fn run_next_auto_atk(&self) -> ServerEvent {
        self.pending_auto_atks.set(self.pending_auto_atks.get() - 1);
        {
            let mut data = self.state.borrow_mut();
            let game_manager = &mut data.core_game_data.game_manager;
            // One boss of a multi-boss round can end the game; the ones queued behind it
            // must not keep swinging afterwards (same guard as the server's
            // `update_core_game_data_after_atk`).
            if game_manager.game_state.status == GameStatus::EndOfGame {
                self.pending_auto_atks.set(0);
            } else {
                let _ = game_manager.launch_attack(None);
            }
        }
        update_event(&self.state)
    }

    pub async fn recv(&self) -> Option<ServerEvent> {
        // `rx` is only ever locked for the duration of a single `.next().await` call,
        // and `recv` is only ever called from GameChannel::recv's one call site
        // (App()'s receive loop) — never concurrently with itself — so `.lock().await`
        // always acquires immediately in practice; it's an async mutex (not a plain
        // `RefCell`) specifically so holding the guard across `.next().await` is sound.
        let mut rx = self.rx.lock().await;
        // Whatever the player's own action already queued goes out first and instantly —
        // only once the UI is up to date with it do the enemy's attacks start landing.
        match rx.try_recv() {
            Ok(event) => return Some(event),
            Err(e) if e.is_closed() => return None, // closed and drained
            Err(_) => {}                            // nothing queued right now
        }
        if self.pending_auto_atks.get() > 0 {
            // Released before the wait: `send` must stay responsive while the enemy round
            // plays out, and nothing else ever needs the receiver meanwhile.
            drop(rx);
            dioxus_sdk_time::sleep(AUTO_ATK_DELAY).await;
            return Some(self.run_next_auto_atk());
        }
        rx.next().await
    }

    fn push(&self, event: ServerEvent) {
        let _ = self.tx.unbounded_send(event);
    }
}

impl Default for LocalChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Handles one `ClientEvent` against local state, returning the `ServerEvent`(s) to
/// report back. Actions outside offline mode's supported subset (multiplayer lobby,
/// shop, admin) are logged and produce no events — a deliberate no-op, not a panic,
/// since the UI paths that would trigger them shouldn't be reachable in offline mode
/// (no "join game" screen, no shop button) but "silently does nothing" is a much safer
/// failure mode than crashing if one is reached anyway.
fn dispatch(state: &Rc<RefCell<ServerData>>, msg: ClientEvent) -> Vec<ServerEvent> {
    match msg {
        ClientEvent::InitializeGame(server_name, player_name, universe, is_single_player) => {
            match crate::local_engine::new_local_game(&universe) {
                Ok(mut core) => {
                    core.is_single_player = is_single_player;
                    // Must match whatever the caller passed as `server_name` here —
                    // the receive loop copies this straight into the app-wide
                    // `SERVER_NAME` signal (main.rs's `UpdateServerData` handler),
                    // which `lobby_page.rs`'s "show the Start Game button" host check
                    // (`SERVER_NAME() == local_login_name_session()`) compares against
                    // `local_login_name_session()`. A previous hardcoded placeholder
                    // here ("offline", disagreeing with whatever name the login flow
                    // actually used) made that check permanently false, silently
                    // hiding the Start Game button for the rest of the session.
                    core.server_name = server_name;
                    // Mirrors the real server's `init_new_game_by_player` ->
                    // `add_server_data_with_player` -> `add_player_to_server`, which
                    // pre-creates an empty `PlayerInfo` entry for the joining player as
                    // soon as the game/lobby exists, before any character is picked.
                    // Without this, `players_info` stays empty and
                    // `character_select.rs`'s `CharacterSelect` — which early-returns
                    // blank whenever `players_info.is_empty()` — never renders anything
                    // to click at all, and the lobby's player count never leaves 0.
                    core.players_nb = 1;
                    let mut data = state.borrow_mut();
                    data.core_game_data = core;
                    data.players_data.owner_player_name = player_name.clone();
                    data.players_data
                        .players_info
                        .entry(player_name)
                        .or_default();
                }
                Err(e) => {
                    dioxus::logger::tracing::error!("offline mode: InitializeGame failed: {e}");
                }
            }
            vec![update_event(state)]
        }

        ClientEvent::AddCharacterOnServerData(_server_name, player_name, character_name) => {
            let mut data = state.borrow_mut();
            if let Err(e) = crate::local_engine::add_hero(&mut data.core_game_data, &character_name)
            {
                dioxus::logger::tracing::error!(
                    "offline mode: AddCharacterOnServerData failed: {e}"
                );
            } else {
                data.core_game_data
                    .heroes_chosen
                    .insert(player_name.clone(), character_name.clone());
                data.players_data
                    .players_info
                    .entry(player_name)
                    .or_default()
                    .character_id_names
                    .push(character_name);
            }
            drop(data);
            vec![update_event(state)]
        }

        ClientEvent::StartGame(_server_name) => {
            let mut data = state.borrow_mut();
            if let Err(e) = crate::local_engine::start_local_game(&mut data.core_game_data) {
                dioxus::logger::tracing::error!("offline mode: StartGame failed: {e}");
            }
            drop(data);
            vec![update_event(state)]
        }

        // Picking an attack: flags every character it could legally reach, which is what
        // `character_page.rs` highlights as selectable. Mirrors the server's
        // `request_set_targeted_characters`.
        //
        // Not optional plumbing: lib-rpg only applies an Individual Enemy/Ally effect to a
        // character whose `is_current_target` is set (`is_effect_applied` in
        // `rounds_information.rs`). While these two events fell through to the
        // unsupported-action catch-all, every hero attack offline landed on nobody at all
        // — no damage, no log line, and no sound, since an attack with no effects has no
        // sound cue to classify.
        ClientEvent::RequestTargetedCharacter(_server_name, launcher_name, atk_name) => {
            let mut data = state.borrow_mut();
            data.core_game_data
                .game_manager
                .pm
                .set_targeted_characters(&launcher_name, &atk_name);
            drop(data);
            vec![update_event(state)]
        }

        // Clicking one of those highlighted characters: narrows the attack down to it.
        // Mirrors the server's `request_set_one_target`.
        ClientEvent::RequestSetOneTarget(_server_name, launcher_name, atk_name, target_name) => {
            let mut data = state.borrow_mut();
            data.core_game_data.game_manager.pm.set_one_target(
                &launcher_name,
                &atk_name,
                &target_name,
            );
            drop(data);
            vec![update_event(state)]
        }

        ClientEvent::LaunchAttack(_server_name, atk_name) => {
            let mut data = state.borrow_mut();
            let _ = data
                .core_game_data
                .game_manager
                .launch_attack(Some(&atk_name));
            drop(data);
            vec![update_event(state)]
        }

        // Mirrors the real server's `set_universe_on_server_data`. `InitializeGame` already
        // applies the universe it was handed, so in the normal flow this arrives carrying the
        // same value and is a no-op — but it is also sent on its own when the universe picker
        // changes, and falling through to the catch-all left `core.universe` and the scenario
        // list stale (logged as "unsupported action SetUniverse").
        ClientEvent::SetUniverse(_server_name, universe) => {
            let mut data = state.borrow_mut();
            match crate::local_engine::scenarios_for_universe(&universe) {
                Ok(filtered) => {
                    let core = &mut data.core_game_data;
                    core.universe = universe;
                    core.game_manager.all_scenarios = filtered.clone();
                    core.game_manager.states_scenarios.clear();
                    for scenario in &filtered {
                        core.game_manager
                            .states_scenarios
                            .insert(scenario.name.clone(), ScenarioState::NotStarted);
                    }
                }
                Err(e) => {
                    dioxus::logger::tracing::error!("offline mode: SetUniverse failed: {e}");
                }
            }
            drop(data);
            vec![update_event(state)]
        }

        ClientEvent::EnterOverworld(_server_name, map_id) => {
            let mut data = state.borrow_mut();
            let hero_id = owner_hero_id(&data);
            if let Err(e) = crate::local_engine::enter_overworld_map(
                &mut data.core_game_data,
                &map_id,
                None,
                hero_id.as_deref(),
            ) {
                dioxus::logger::tracing::error!("offline mode: EnterOverworld failed: {e}");
            }
            drop(data);
            vec![update_event(state)]
        }

        ClientEvent::MovePlayer(_server_name, player_name, dir, lang) => {
            let mut data = state.borrow_mut();
            // Real server: only the server owner controls the party sprite.
            if player_name != data.players_data.owner_player_name {
                return Vec::new();
            }
            let Some(hero_id) = owner_hero_id(&data) else {
                dioxus::logger::tracing::warn!("offline mode: MovePlayer with no hero");
                return Vec::new();
            };
            let lang = crate::common::lang_from_app_lang(&lang);
            if let Err(e) =
                crate::local_engine::move_player(&mut data.core_game_data, &hero_id, dir, lang)
            {
                dioxus::logger::tracing::error!("offline mode: MovePlayer failed: {e}");
            }
            drop(data);
            vec![update_event(state)]
        }

        ClientEvent::Interact(_server_name, player_name, lang) => {
            let mut data = state.borrow_mut();
            if player_name != data.players_data.owner_player_name {
                return Vec::new();
            }
            let Some(hero_id) = owner_hero_id(&data) else {
                dioxus::logger::tracing::warn!("offline mode: Interact with no hero");
                return Vec::new();
            };
            let lang = crate::common::lang_from_app_lang(&lang);
            if let Err(e) = crate::local_engine::interact(&mut data.core_game_data, &hero_id, lang)
            {
                dioxus::logger::tracing::error!("offline mode: Interact failed: {e}");
            }
            drop(data);
            vec![update_event(state)]
        }

        ClientEvent::DismissDialog(_server_name, player_name) => {
            let mut data = state.borrow_mut();
            if player_name != data.players_data.owner_player_name {
                return Vec::new();
            }
            crate::local_engine::dismiss_dialog(&mut data.core_game_data);
            drop(data);
            vec![update_event(state)]
        }

        ClientEvent::ExitOverworld(_server_name) => {
            let mut data = state.borrow_mut();
            crate::local_engine::exit_overworld(&mut data.core_game_data);
            drop(data);
            vec![update_event(state)]
        }

        other => {
            dioxus::logger::tracing::warn!("offline mode: unsupported action {other:?}");
            Vec::new()
        }
    }
}

fn update_event(state: &Rc<RefCell<ServerData>>) -> ServerEvent {
    ServerEvent::UpdateServerData(Box::new(state.borrow().clone()))
}

/// The owner's first chosen hero id — the one overworld sprite represents the whole
/// party by. Matches the real server's `players_info.get(&owner_name).and_then(|info|
/// info.character_id_names.first())` lookup used throughout `event.rs`'s overworld
/// handlers.
fn owner_hero_id(data: &ServerData) -> Option<String> {
    data.players_data
        .players_info
        .get(&data.players_data.owner_player_name)
        .and_then(|info| info.character_id_names.first().cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sfx_cue::Sfx;
    use lib_rpg::server::game_manager::ResultLaunchAttack;
    use lib_rpg::server::server_manager::GamePhase;

    /// End-to-end proof of the actual pipeline the UI will drive: activate, then send
    /// the same ClientEvent sequence character-select → StartGame → LaunchAttack goes
    /// through for real, reading state back only via `recv()` — exactly as `GameChannel`
    /// and the App() receive loop will, never touching `LocalChannel`'s internals.
    #[test]
    fn local_channel_full_session_round_trip() {
        let channel = LocalChannel::new();
        channel.activate();

        let init = futures::executor::block_on(channel.recv()).expect("InitClient");
        let ServerEvent::InitClient(_, heroes) = init else {
            panic!("expected InitClient first, got {init:?}");
        };
        let hero_name = heroes
            .iter()
            .find(|h| h.universe == "lotr")
            .expect("at least one lotr hero")
            .db_full_name
            .clone();

        channel.send(ClientEvent::InitializeGame(
            LOCAL_PLAYER_NAME.to_owned(),
            LOCAL_PLAYER_NAME.to_owned(),
            "lotr".to_owned(),
            true,
        ));
        let after_init = expect_update(&channel);
        assert_eq!(after_init.core_game_data.game_phase, GamePhase::InitGame);
        // `character_select.rs`'s `CharacterSelect` renders nothing at all whenever
        // `players_info` is empty, so this entry must exist *before* any character is
        // picked — not just after, which `AddCharacterOnServerData`'s own `.entry(...)`
        // would paper over even if `InitializeGame` never created it.
        assert!(
            after_init
                .players_data
                .players_info
                .contains_key(LOCAL_PLAYER_NAME),
            "expected an empty PlayerInfo for {LOCAL_PLAYER_NAME:?} right after InitializeGame, got {:?}",
            after_init.players_data.players_info
        );

        channel.send(ClientEvent::AddCharacterOnServerData(
            LOCAL_PLAYER_NAME.to_owned(),
            LOCAL_PLAYER_NAME.to_owned(),
            hero_name.clone(),
        ));
        let after_add = expect_update(&channel);
        assert_eq!(
            after_add.core_game_data.game_manager.pm.active_heroes.len(),
            1
        );

        channel.send(ClientEvent::StartGame(LOCAL_PLAYER_NAME.to_owned()));
        let after_start = expect_update(&channel);
        assert_eq!(after_start.core_game_data.game_phase, GamePhase::Running);

        let atk_name = after_start
            .core_game_data
            .game_manager
            .pm
            .current_player
            .attacks_list
            .keys()
            .next()
            .cloned()
            .expect("hero should have at least one attack");
        channel.send(ClientEvent::LaunchAttack(
            LOCAL_PLAYER_NAME.to_owned(),
            atk_name,
        ));
        let after_attack = expect_update(&channel);
        // Not asserting a specific effect here (same "not every attack lands on a
        // fresh single-hero party" caveat as local_engine's own combat test) — this
        // test's job is proving the channel plumbing round-trips real state, not
        // re-proving combat math local_engine::tests already covers.
        assert_eq!(after_attack.core_game_data.server_name, LOCAL_PLAYER_NAME);
    }

    /// The enemy actually gets its turn offline. Before this, `dispatch` ran the
    /// player's attack and stopped there — the server-side `process_ennemy_atk` step
    /// had no offline counterpart, so a boss never once hit back in local mode.
    #[test]
    fn the_enemy_round_is_played_after_the_hero_attacks() {
        let channel = started_session();
        let after_start = channel.state.borrow().clone();

        let hero_id = after_start
            .core_game_data
            .game_manager
            .pm
            .current_player
            .id_name
            .clone();
        let atk_name = after_start
            .core_game_data
            .game_manager
            .pm
            .current_player
            .attacks_list
            .keys()
            .next()
            .cloned()
            .expect("hero should have at least one attack");
        channel.send(ClientEvent::LaunchAttack(
            LOCAL_PLAYER_NAME.to_owned(),
            atk_name,
        ));

        // The player's own attack is still reported first and without any delay.
        let after_attack = expect_update(&channel);
        assert_eq!(
            after_attack
                .core_game_data
                .game_manager
                .game_state
                .last_result_atk
                .launcher_id_name,
            hero_id
        );
        // ...and the boss whose turn it now is has been queued to answer it.
        assert!(
            channel.pending_auto_atks.get() > 0,
            "expected the boss round to be queued, order_to_play={:?}",
            after_attack
                .core_game_data
                .game_manager
                .game_state
                .order_to_play
        );

        // Stepping the queue directly rather than through `recv`, which would sit out
        // the real AUTO_ATK_DELAY between attacks — the pacing is a UI concern, what
        // matters here is that the attack happens at all and comes from the enemy.
        let ServerEvent::UpdateServerData(after_boss) = channel.run_next_auto_atk() else {
            panic!("an enemy attack should report a full state update");
        };
        let boss_atk = &after_boss
            .core_game_data
            .game_manager
            .game_state
            .last_result_atk;
        assert!(
            boss_atk.is_boss_atk,
            "expected a boss attack, got {boss_atk:?}"
        );
        assert_ne!(boss_atk.launcher_id_name, hero_id);
        assert_eq!(channel.pending_auto_atks.get(), 0);
    }

    /// One `Charge`, cast in a fresh fight, returning the cues it produced.
    ///
    /// A fresh session each time keeps the boss alive and the state clean, so the
    /// only thing that varies between calls is the combat roll itself.
    fn cast_charge_in_a_fresh_fight() -> (ResultLaunchAttack, Vec<Sfx>) {
        let channel = started_session();
        let hero = channel
            .state
            .borrow()
            .core_game_data
            .game_manager
            .pm
            .current_player
            .id_name
            .clone();
        channel.send(ClientEvent::RequestTargetedCharacter(
            LOCAL_PLAYER_NAME.to_owned(),
            hero,
            "Charge".to_owned(),
        ));
        let _ = expect_update(&channel);
        channel.send(ClientEvent::LaunchAttack(
            LOCAL_PLAYER_NAME.to_owned(),
            "Charge".to_owned(),
        ));
        let after = expect_update(&channel);
        let ra = after
            .core_game_data
            .game_manager
            .game_state
            .last_result_atk
            .clone();
        let cues = crate::sfx_cue::classify_attack(&ra);
        (ra, cues)
    }

    /// True for a cast the enemy avoided. Those are a different event and are
    /// supposed to sound different, so the tests below skip them rather than
    /// comparing them against a cast that connected.
    fn was_avoided(cues: &[Sfx]) -> bool {
        matches!(cues, [Sfx::Dodge] | [Sfx::Block])
    }

    /// A hero attack must actually reach the enemy offline — and therefore make a
    /// sound. Regression test for `RequestTargetedCharacter`/`RequestSetOneTarget`
    /// falling through to the unsupported-action catch-all: with no character ever
    /// flagged as the current target, lib-rpg skipped every Individual effect, so
    /// offline attacks landed nothing and `classify_attack` had no cue to play.
    #[test]
    fn a_hero_attack_lands_and_has_a_sound_cue() {
        for _ in 0..8 {
            let (ra, cues) = cast_charge_in_a_fresh_fight();
            if was_avoided(&cues) {
                continue; // dodged; try again rather than assert on a miss
            }
            assert!(
                !ra.new_game_atk_effects.is_empty(),
                "Charge should land on the targeted enemy, got {ra:?}"
            );
            assert!(
                !cues.is_empty(),
                "a landed attack must have a sound cue, got {ra:?}"
            );
            return;
        }
        panic!("eight casts of Charge in a row were all dodged, which should not happen");
    }

    /// The contract the sound design rests on: one attack, one sound. The same
    /// attack cast in fight after fight must lead with the same family cue — the
    /// sound that gives it its identity — even though the numbers underneath move
    /// from cast to cast as criticals, armour and the HP cap change what lands.
    ///
    /// A critical adds its accent after the family cue, which is the one deliberate
    /// variation.
    #[test]
    fn one_attack_sounds_the_same_on_every_cast() {
        let mut landed = Vec::new();
        for _ in 0..8 {
            let (_, cues) = cast_charge_in_a_fresh_fight();
            if !was_avoided(&cues) {
                landed.push(cues);
            }
        }

        assert!(
            landed.len() >= 2,
            "expected at least two landed casts to compare, got {landed:?}"
        );
        assert!(
            landed.iter().all(|cues| cues.first() == landed[0].first()),
            "Charge sounded different from one cast to the next: {landed:?}"
        );
        assert!(
            landed
                .iter()
                .all(|cues| cues[1..].iter().all(|cue| *cue == Sfx::CriticalHit)),
            "the only cue allowed on top of the family is the crit accent: {landed:?}"
        );
    }

    /// An offline session driven up to the first hero's turn, through the same
    /// ClientEvent sequence the UI sends.
    fn started_session() -> LocalChannel {
        let channel = LocalChannel::new();
        channel.activate();

        let init = futures::executor::block_on(channel.recv()).expect("InitClient");
        let ServerEvent::InitClient(_, heroes) = init else {
            panic!("expected InitClient first, got {init:?}");
        };
        let hero_name = heroes
            .iter()
            .find(|h| h.universe == "lotr")
            .expect("at least one lotr hero")
            .db_full_name
            .clone();
        channel.send(ClientEvent::InitializeGame(
            LOCAL_PLAYER_NAME.to_owned(),
            LOCAL_PLAYER_NAME.to_owned(),
            "lotr".to_owned(),
            true,
        ));
        let _ = expect_update(&channel);
        channel.send(ClientEvent::AddCharacterOnServerData(
            LOCAL_PLAYER_NAME.to_owned(),
            LOCAL_PLAYER_NAME.to_owned(),
            hero_name,
        ));
        let _ = expect_update(&channel);
        channel.send(ClientEvent::StartGame(LOCAL_PLAYER_NAME.to_owned()));
        let _ = expect_update(&channel);
        channel
    }

    fn expect_update(channel: &LocalChannel) -> ServerData {
        match futures::executor::block_on(channel.recv()).expect("a ServerEvent") {
            ServerEvent::UpdateServerData(data) => *data,
            other => panic!("expected UpdateServerData, got {other:?}"),
        }
    }
}

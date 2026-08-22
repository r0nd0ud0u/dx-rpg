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

use std::{cell::RefCell, rc::Rc};

use futures::{StreamExt, channel::mpsc, lock::Mutex as AsyncMutex};
use lib_rpg::server::server_manager::ServerData;

use crate::websocket_handler::event::{ClientEvent, ServerEvent};

// `pub(crate)`: the "Play Offline" entry point (login_page.rs) needs this exact value
// too — it must set `local_login_name_session` and `SERVER_NAME` (via
// `send_initialize_game`'s `user_name` argument) to the same string this module uses
// for `owner_player_name`, or the several `owner_player_name == local_login_name_session()`
// host-only-controls checks scattered through startgame_page.rs (load next scenario,
// save game, ...) would never pass during an offline session.
pub(crate) const LOCAL_PLAYER_NAME: &str = "Player";
const LOCAL_CLIENT_ID: u32 = 0;

#[derive(Clone)]
pub struct LocalChannel {
    tx: mpsc::UnboundedSender<ServerEvent>,
    // An async-aware lock, not `RefCell`: `recv` needs to hold this across an `.await`
    // (for the whole duration of `.next().await`), which a plain `RefCell`'s guard
    // isn't safe to do (it can't yield to other tasks, so a would-be second borrow
    // panics instead of just waiting its turn).
    rx: Rc<AsyncMutex<mpsc::UnboundedReceiver<ServerEvent>>>,
    state: Rc<RefCell<ServerData>>,
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
        }
    }

    /// Starts an offline single-player session: registers embedded game data, resets
    /// local state, and pushes the same `InitClient` a real server sends right after a
    /// websocket connects — so the character-select UI (which reads the hero list from
    /// that event) works unchanged. Call once, when the user picks "Play Offline".
    pub fn activate(&self) {
        crate::embedded_data::register();
        *self.state.borrow_mut() = ServerData::default();

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
        for event in dispatch(&self.state, msg) {
            self.push(event);
        }
    }

    pub async fn recv(&self) -> Option<ServerEvent> {
        // `rx` is only ever locked for the duration of a single `.next().await` call,
        // and `recv` is only ever called from GameChannel::recv's one call site
        // (App()'s receive loop) — never concurrently with itself — so `.lock().await`
        // always acquires immediately in practice; it's an async mutex (not a plain
        // `RefCell`) specifically so holding the guard across `.next().await` is sound.
        self.rx.lock().await.next().await
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
        ClientEvent::InitializeGame(server_name, _player_name, universe, is_single_player) => {
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
                    let mut data = state.borrow_mut();
                    data.core_game_data = core;
                    data.players_data.owner_player_name = LOCAL_PLAYER_NAME.to_owned();
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

        ClientEvent::LaunchAttack(_server_name, atk_name) => {
            let mut data = state.borrow_mut();
            let _ = data
                .core_game_data
                .game_manager
                .launch_attack(Some(&atk_name));
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

#[cfg(test)]
mod tests {
    use super::*;
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

    fn expect_update(channel: &LocalChannel) -> ServerData {
        match futures::executor::block_on(channel.recv()).expect("a ServerEvent") {
            ServerEvent::UpdateServerData(data) => *data,
            other => panic!("expected UpdateServerData, got {other:?}"),
        }
    }
}

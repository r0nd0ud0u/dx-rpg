//! Client-only: makes the game data under `offlines/` (embedded at compile time by
//! `build.rs`) available via `lib_rpg::utils::set_embedded_files`, so
//! `DataManager::try_new(OFFLINE_PATH)` works the same way on a client build as it does
//! on the server (which reads `offlines/` straight off the real filesystem instead).
//!
//! Not compiled into the server build at all — `build.rs` skips generating
//! `embedded_offline_files.rs` there too (the server never needs this; see its own
//! comment), so this module couldn't compile there even if included.
#![cfg(not(feature = "server"))]

include!(concat!(env!("OUT_DIR"), "/embedded_offline_files.rs"));

/// Registers the embedded `offlines/` data with lib-rpg. Call once, before any code that
/// might construct a `DataManager` (offline mode's local game engine). Idempotent —
/// `set_embedded_files` itself is first-caller-wins, so a repeat call is a harmless no-op.
pub fn register() {
    let files = EMBEDDED_OFFLINE_FILES
        .iter()
        .map(|(key, content)| (key.to_string(), *content))
        .collect();
    lib_rpg::utils::set_embedded_files(files);
}

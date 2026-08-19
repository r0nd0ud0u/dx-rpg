use lib_rpg::server::server_manager::ServerManager;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex, MutexGuard};

/// server only: shared game state
pub static SERVER_MANAGER: Lazy<Arc<Mutex<ServerManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(ServerManager::default())));

/// Locks `SERVER_MANAGER`, recovering the guard if the mutex is poisoned
/// instead of panicking. A panic in any one event handler while holding this
/// lock would otherwise poison it permanently — every subsequent
/// `.lock().unwrap()` anywhere in the server then panics immediately too,
/// freezing the game for every player, not just the one whose action first
/// panicked, until the server process is restarted.
pub fn lock_server_manager() -> MutexGuard<'static, ServerManager> {
    SERVER_MANAGER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

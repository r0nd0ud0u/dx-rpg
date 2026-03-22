use lib_rpg::server::server_manager::ServerManager;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

/// server only: shared game state
pub static SERVER_MANAGER: Lazy<Arc<Mutex<ServerManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(ServerManager::default())));

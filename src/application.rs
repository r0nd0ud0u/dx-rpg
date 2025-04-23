use std::time::Duration;

use dioxus::prelude::server_fn;
use dioxus::{prelude::server, prelude::ServerFnError};
use lib_rpg::game_manager::GameManager;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Application {
    pub game_manager: GameManager,
}

#[server]
pub async fn try_new() -> Result<Application, ServerFnError> {
    match GameManager::try_new("") {
        Ok(gm) => Ok(Application { game_manager: gm }),
        Err(_) => Err(ServerFnError::Request(
            "Failed to create GameManager".to_string(),
        )),
    }
}

#[server]
pub async fn sleep_from_millis(millis: u64) -> Result<(), ServerFnError> {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(Duration::from_millis(millis)).await;
    Ok(())
}

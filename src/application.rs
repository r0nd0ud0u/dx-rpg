use std::time::Duration;

use dioxus::prelude::*;
use dioxus::{prelude::server, prelude::ServerFnError};
use lib_rpg::game_manager::GameManager;
use serde::Deserialize;
use serde::Serialize;
use colorgrad::Gradient;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Application {
    pub game_manager: GameManager,
    #[serde(skip)]
    pub gradient: Option<colorgrad::CatmullRomGradient>,
}

impl PartialEq for Application {
    fn eq(&self, other: &Self) -> bool {
        self.game_manager == other.game_manager
        // `gradient` is intentionally excluded
    }
}

#[server]
pub async fn try_new() -> Result<Application, ServerFnError> {
    let g: colorgrad::CatmullRomGradient = colorgrad::GradientBuilder::new()
        .html_colors(&["deeppink", "gold", "seagreen"])
        .build::<colorgrad::CatmullRomGradient>()?;
    match GameManager::try_new("") {
        Ok(gm) => Ok(Application { game_manager: gm, gradient: Some(g) }),
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

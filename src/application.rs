use anyhow::Result;
use lib_rpg::game_manager::GameManager;

#[derive(Default, Debug, Clone)]
pub struct Application {
    pub game_manager: GameManager,
}

impl Application {
    pub fn try_new() -> Result<Application> {
        let gm = GameManager::try_new()?;
        Ok(Application { game_manager: gm })
    }
}

use lib_rpg::game_manager::GameManager;

#[derive(Default, Debug, Clone)]
pub struct Application {
    pub game_manager: GameManager,
}

impl Application {
    pub fn try_new() -> Result<Application, String> {
        match GameManager::try_new() {
            Ok(gm) => Ok(Application { game_manager: gm }),
            Err(e) => Err(format!("Failed to create GameManager: {}", e)),
        }
    }
}

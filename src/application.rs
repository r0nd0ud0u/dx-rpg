use rust_rpg::game_manager::GameManager;

#[derive(Default, Debug, Clone)]
pub struct Application {
    pub game_manager: GameManager,
}

impl Application {
    pub fn init(self) {}
}

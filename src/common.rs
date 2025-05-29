use dioxus::prelude::*;

use crate::application::Application;
use colorgrad::{CatmullRomGradient, GradientBuilder};
use once_cell::sync::Lazy;

pub static APP: GlobalSignal<Application> = Signal::global(Application::default);

pub static ENERGY_GRAD: Lazy<CatmullRomGradient> = Lazy::new(|| {
    GradientBuilder::new()
        .html_colors(&["deeppink", "gold", "seagreen"])
        .build::<CatmullRomGradient>()
        .expect("Failed to build gradient")
});

pub mod tempo_const {
    pub const AUTO_ATK: u64 = 3000;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PageStatus {
    HomePage = 0,
    NewGame,
    LoadGame,
}

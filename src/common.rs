use dioxus::prelude::*;

use crate::application::Application;

pub static APP: GlobalSignal<Application> = Signal::global(Application::default);

//pub static GLOBAL_RES: GlobalSignal<GlobalResource> = Signal::global(GlobalResource::default);

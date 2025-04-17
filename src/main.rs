use dioxus::prelude::*;
use dx_rpg::{
    application::{self, Application},
    character_page,
};
use lib_rpg::common::stats_const::HP;
use lib_rpg::testing_atk;
use lib_rpg::testing_target;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

static APP: GlobalSignal<Application> = Signal::global(|| Application::default());

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn GameBoard() -> Element {
    rsx! {
        div { class: "grid-board",
            div {
                for c in APP.read().game_manager.pm.active_heroes.iter() {
                    character_page::CharacterPanel { c: c.clone() }
                }
            }
            div {
                // add containers
            }
            div {
                for c in APP.read().game_manager.pm.active_bosses.iter() {
                    character_page::CharacterPanel { c: c.clone() }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ButtonStatus {
    StartGame = 0,
    StartRound,
    ValidateAction,
}

/// Home page
#[component]
fn Home() -> Element {
    let mut state = use_signal(|| ButtonStatus::StartGame);
    rsx! {
        if state() == ButtonStatus::StartGame {
            button {
                onclick: move |_| async move {
                    println!("component found");
                    match application::try_new().await {
                        Ok(app) => *APP.write() = app,
                        Err(_) => println!("no app"),
                    }
                    let _ = APP.write().game_manager.start_new_turn();
                    state.set(ButtonStatus::ValidateAction);
                },
                "Start"
            }
        }
        if state() == ButtonStatus::ValidateAction {
            button {
                onclick: move |_| async move {
                    APP.write()
                        .game_manager
                        .launch_attack(
                            "SimpleAtk",
                            vec![testing_target::build_target_angmar_indiv()],
                        );
                },
                "launch atk"
            }
            button {
                onclick: move |_| async move {
                    if !APP.write().game_manager.new_round() {
                        APP.write().game_manager.start_new_turn();
                        state.set(ButtonStatus::StartRound);
                    }
                },
                "Inventory"
            }
        }
        GameBoard {}
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {}, "Home" }
        }

        Outlet::<Route> {}
    }
}

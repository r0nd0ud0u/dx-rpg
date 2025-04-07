use dioxus::prelude::*;
use dx_rpg::{
    application::{self, Application},
    character_page,
};
use lib_rpg::common::stats_const::HP;
use lib_rpg::testing_target;
use lib_rpg::testing_atk;

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

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        button {
            onclick: move |_| async move {
                println!("component found");
                match application::try_new().await {
                    Ok(app) => *APP.write() = app,
                    Err(_) => println!("no app"),
                }
            },
            "Start"
        }
        button {
            onclick: move |_| async move {
                APP.write().game_manager.start_new_turn();
            },
            "Start new turn"
        }
        button {
            onclick: move |_| async move {
                APP.write().game_manager.new_round();
            },
            "Start new round"
        }
        button {
            onclick: move |_| async move {
                let atk = testing_atk::build_atk_berseck_damage1();
                if APP.write().game_manager.pm.current_player.attacks_list.is_empty() {APP.write().game_manager.pm.current_player.attacks_list.insert(atk.name.clone(), atk);   }
                APP.write().game_manager.launch_attack("atk1", vec![testing_target::build_target_angmar_indiv()]);
            },
            "launch atk"
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

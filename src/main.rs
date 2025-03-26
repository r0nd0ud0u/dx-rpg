use dioxus::prelude::*;
use dx_rpg::{
    application::{self, Application},
    character_page,
};
use lib_rpg::common::stats_const::HP;

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
                for c in APP.read().game_manager.player_manager.all_heroes.iter() {
                    character_page::CharacterPanel { c: c.clone() }
                }
            }
            div {
                button {
                    onclick: move |_| {
                        async move {
                            APP.write()
                                .game_manager
                                .player_manager
                                .all_bosses
                                .iter_mut()
                                .for_each(|b| {
                                    b.stats.all_stats.get_mut(HP).unwrap().current = b
                                        .stats
                                        .all_stats
                                        .get_mut(HP)
                                        .unwrap()
                                        .current
                                        .saturating_sub(10);
                                });
                            println!("boss not foundyet ");
                        }
                    },
                    "add damagesss"
                }
            }
            div {
                for c in APP.read().game_manager.player_manager.all_bosses.iter() {
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

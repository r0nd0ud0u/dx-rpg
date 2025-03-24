use dioxus::prelude::*;
use dx_rpg::{
    application::{self, Application},
    character_page,
};

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
    let resource = use_resource(move || async move {
        match application::try_new().await {
            Ok(app) => *APP.write() = app,
            Err(_) => println!("no app"),
        }
    });
    match resource() {
        Some(_) => {
            rsx! {
                document::Link { rel: "icon", href: FAVICON }
                document::Link { rel: "stylesheet", href: MAIN_CSS }
                Router::<Route> {}
            }
        }
        _ => rsx! {},
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
            div {}
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

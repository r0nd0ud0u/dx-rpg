use std::sync::OnceLock;

use dioxus::prelude::*;
use dx_rpg::{application::Application, character_page, game_page};
use lib_rpg::character::Character;

static CHARACTERS: GlobalSignal<Vec<Character>> = Signal::global(Vec::new);


#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/game/:id")]
    Game { id: String },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn application() -> &'static Application {
    static APP_MANAGER: OnceLock<Application> = OnceLock::new();
    APP_MANAGER.get_or_init(|| Application::try_new().expect("Failed to initialize application"))
}

//pub(crate) static APP_MANAGER: GlobalSignal<Application> = GlobalSignal::new(Application::default);

fn main() {
    // *APP_MANAGER.write() = Application::try_new().expect("Failed to initialize application");
    /* println!(
        "heroes:{:?}",
        APP_MANAGER.read().game_manager.player_manager.all_heroes[0].name
    ); */
    println!(
        "heroes:{:?}",
        application().game_manager.player_manager.all_heroes[0].name
    );
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
fn Game(id: String) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Host" }
        game_page::Game_page { id }
    }
}

#[component]
fn Hero(name: String) -> Element {
/*     println!(
        "heroes test:{:?}",
        application().game_manager.player_manager.all_heroes[0].name
    );
    for c in application().game_manager.player_manager.all_heroes.clone() {
        CHARACTERS.write().push(c);
    }
    rsx! {
        for c in CHARACTERS.read().iter() {
             character_page::Character_page{name:c.name.clone()}
         }
     } */
      rsx!{
        character_page::Character_page{name}
      }    
}

#[component]
fn WeatherElement(weather: String) -> Element {
    rsx! { p { "The weather is {weather}" } }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Game { id: "" }
        //Echo {}
        Hero { name: "Dracaufeu" }
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div { id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p {
                "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components."
            }

            // Navigation links
            Link { to: Route::Blog { id: id - 1 }, "Previous" }
            span { " <---> " }
            Link { to: Route::Blog { id: id + 1 }, "Next" }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::Blog { id: 1 }, "Blog" }
        }

        Outlet::<Route> {}
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(String::new);

    rsx! {
        div { id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput: move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[server(EchoServer)]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

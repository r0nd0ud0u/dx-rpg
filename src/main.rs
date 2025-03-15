use dioxus::prelude::*;
use dx_rpg::{application::Application, character_page, game_page};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:nb")]
    Blog { nb: i32 },
    #[route("/game/:game")]
    Game { game: String },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Clone)]
struct MyState {
    app: Signal<Application>,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    /*     match Application::try_new() {
        Ok(app) => {
            use_context_provider(|| MyState { app: Signal::new(app) });
            rsx! {
                document::Link { rel: "icon", href: FAVICON }
                document::Link { rel: "stylesheet", href: MAIN_CSS }
                Router::<Route> {}
            }
        }
        _ => rsx! { "no app" },
    } */
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Game(game: String) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Host" }
        game_page::Game_page { game }
    }
}

#[component]
fn Hero() -> Element {
    let signalApp: Signal<Application> = use_context();
    rsx! {
        for c in signalApp.read().game_manager.player_manager.all_heroes.iter() {
            character_page::Character_page { name: c.name.clone() }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Game { game: "" }
        Echo {}
    }
}

/// Blog page
#[component]
pub fn Blog(nb: i32) -> Element {
    rsx! {
        div { id: "blog",

            // Content
            h1 { "This is blog #{nb}!" }
            p {
                "In blog #{nb}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components."
            }

            // Navigation links
            Link { to: Route::Blog { nb: nb - 1 }, "Previous" }
            span { " <---> " }
            Link { to: Route::Blog { nb: nb + 1 }, "Next" }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::Blog { nb: 1 }, "Blog" }
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

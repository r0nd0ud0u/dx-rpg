use dioxus::prelude::*;
use dx_rpg::common::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Init logger
    println!("starting app");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css"),
        }
        Router::<Route> {}
    }
}

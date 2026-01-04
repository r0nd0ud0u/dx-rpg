use dioxus::{logger::tracing::Level, prelude::*};
use dx_rpg::common::{Route, DX_COMP_CSS};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Init logger
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMP_CSS }
        Router::<Route> {}
    }
}

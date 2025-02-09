use dioxus::prelude::*;

#[component]
pub fn Game_page(id: String) -> Element {
    rsx! {
        div { id: "hero" }
    }
}

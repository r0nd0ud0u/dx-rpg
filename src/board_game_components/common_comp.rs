use dioxus::prelude::*;

#[component]
pub fn ButtonLink(
    target: NavigationTarget,
    name: String,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div { class: "button-link",
            Link { class: "header-text", to: target, onclick, "{name}" }
        }
    }
}

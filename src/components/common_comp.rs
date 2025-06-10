use dioxus::prelude::*;

#[component]
pub fn ButtonLink(target: NavigationTarget, name: String) -> Element {
    rsx! {
        div { class: "button-link",
            Link { class: "header-text", to: target, "{name}" }
        }
    }
}

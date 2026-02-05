use dioxus::prelude::*;

#[component]
pub fn ButtonLink(
    target: NavigationTarget,
    name: String,
    #[props(extends=button)] attributes: Vec<Attribute>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "button-link",
            Link { class: "header-text", to: target, "{name}" }
        }
    }
}

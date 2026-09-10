use dioxus::prelude::*;
use dioxus_i18n::t;

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

/// "Back" affordance for top-level pages, always top-left above the title, always a single
/// click to an explicit `target` (use `BackHistoryButton` when the right destination
/// depends on how the player arrived). No confirmation — pass `onclick` for cleanup that
/// must run first, e.g. leaving a tentatively-joined server. `RunningGamePage` has none:
/// its quit flow needs the confirmation this skips.
#[component]
pub fn BackButton(target: NavigationTarget, onclick: Option<EventHandler<MouseEvent>>) -> Element {
    rsx! {
        Link { class: "back-button", to: target, onclick,
            span { class: "back-button-icon", "←" }
            {t!("common-back")}
        }
    }
}

/// Back control that steps through the router's history instead of a fixed route, for
/// pages reachable from several places — the lobby is entered from create-server, the
/// join list and the offline card, and any fixed target is wrong for two of them.
#[component]
pub fn BackHistoryButton(onclick: Option<EventHandler<MouseEvent>>) -> Element {
    let navigator = use_navigator();
    rsx! {
        button {
            class: "back-button",
            r#type: "button",
            onclick: move |e: MouseEvent| {
                if let Some(handler) = onclick {
                    handler.call(e);
                }
                navigator.go_back();
            },
            span { class: "back-button-icon", "←" }
            {t!("common-back")}
        }
    }
}

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

/// Consistent "back" affordance for every top-level page reached by navigating forward
/// from somewhere else (Home, CreateServer, JoinOngoingGame, AdminPage, and the
/// lobby's pre-game screen). Always in the same spot (top-left, above the page title) and
/// always a single click straight to an explicit `target` route — not history-based
/// "go back" (which could land on a stale intermediate state depending on how the player
/// arrived), and no confirmation dialog, since every call site only wires this up where
/// leaving has no destructive side effect (pass `onclick` for any cleanup that does need
/// to run first, e.g. disconnecting from a server that was only tentatively joined).
/// `RunningGamePage` deliberately has no `BackButton`: it already has its own quit flow
/// (`QuitGameButton` / Navbar's Quit-game dialog), which needs the confirmation step this
/// button intentionally skips.
#[component]
pub fn BackButton(target: NavigationTarget, onclick: Option<EventHandler<MouseEvent>>) -> Element {
    rsx! {
        Link { class: "back-button", to: target, onclick,
            span { class: "back-button-icon", "←" }
            {t!("common-back")}
        }
    }
}

/// Back control that steps through the router's own history rather than pointing at
/// one fixed route.
///
/// For pages reachable from more than one place, where the right destination is
/// wherever the player actually came from: the lobby, for instance, is reached from
/// the create-server page, the join-game list, and the offline card, and a fixed
/// target sends two of those three somewhere the player has never been.
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

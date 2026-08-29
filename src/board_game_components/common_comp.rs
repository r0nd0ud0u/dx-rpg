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
/// from somewhere else (Home, CreateServer, LoadGame, JoinOngoingGame, AdminPage, and the
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

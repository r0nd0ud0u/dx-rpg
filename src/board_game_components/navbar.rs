use dioxus::{logger::tracing, prelude::*};
use dioxus_primitives::label::Label;

use crate::{
    auth::server_fn::logout,
    common::{disconnected_user, Route, USER_NAME},
    components::button::{Button, ButtonVariant},
};

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    let navigator = use_navigator();
    rsx! {
        div { class: "navbar",
            div { style: "display: flex; gap: 1rem;",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::AdminPage {}, "Admin" }
            }
            div {
                Button {
                    variant: if USER_NAME() == disconnected_user() { ButtonVariant::Secondary } else { ButtonVariant::Destructive },
                    onclick: move |_| async move {
                        if USER_NAME() != disconnected_user() {
                            match logout().await {
                                Ok(_) => tracing::info!("{} is logged out", USER_NAME()),
                                Err(_) => tracing::info!("Error on {} logout", USER_NAME()),
                            }
                            *USER_NAME.write() = disconnected_user();
                            match logout().await {
                                Ok(_) => tracing::info!(""),
                                Err(_) => tracing::info!(""),
                            }
                        }
                        navigator.push(Route::Home {});
                    },
                    if USER_NAME() == disconnected_user() {
                        "Sign in"
                    } else {
                        "Sign out"
                    }
                }
                Label { html_for: "navbar", "({USER_NAME.read()})" }
            }
        }

        Outlet::<Route> {}
    }
}

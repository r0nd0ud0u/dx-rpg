use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use dioxus_primitives::label::Label;

use crate::{
    auth_manager::server_fn::logout,
    common::{Route, USER_NAME, disconnected_user, is_admin},
    components::button::{Button, ButtonVariant},
    websocket_handler::event::{ClientEvent, ServerEvent},
};

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // socket
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    // nav
    let navigator = use_navigator();
    rsx! {
        div { class: "navbar",
            div { style: "display: flex; gap: 1rem;",
                Link { to: Route::Home {}, "Home" }
                if is_admin() {
                    Link { to: Route::AdminPage {}, "Admin" }
                }
            }
            div {
                Button {
                    variant: if USER_NAME() == disconnected_user() { ButtonVariant::Secondary } else { ButtonVariant::Destructive },
                    onclick: move |_| async move {

                        if USER_NAME() != disconnected_user() {
                            match logout().await {
                                Ok(_) => {
                                    tracing::info!("{} is logged out", USER_NAME());
                                    let _ = socket

                                        .clone()
                                        .send(ClientEvent::Disconnect(USER_NAME()))
                                        .await;
                                }
                                Err(_) => tracing::info!("Error on {} logout", USER_NAME()),
                            }
                            *USER_NAME.write() = disconnected_user();
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

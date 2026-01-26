use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use dioxus_primitives::label::Label;

use crate::{
    auth_manager::server_fn::logout,
    common::{Route, disconnected_user},
    components::button::{Button, ButtonVariant},
    websocket_handler::event::{ClientEvent, ServerEvent},
};

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // socket
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    // local storage
    let mut local_login_session = use_context::<Signal<String>>();
    // nav
    let navigator = use_navigator();

    // snapshot
    let snap_local_login_session = local_login_session();
    rsx! {
        div { class: "navbar",
            div { style: "display: flex; gap: 1rem;",
                Link { to: Route::Home {}, "Home" }
                if snap_local_login_session == "Admin".to_owned() {
                    Link { to: Route::AdminPage {}, "Admin" }
                }
            }
            div {
                Button {
                    variant: if snap_local_login_session == disconnected_user() { ButtonVariant::Secondary } else { ButtonVariant::Destructive },
                    onclick: move |_| async move {
                        if local_login_session() != disconnected_user() {
                            match logout().await {
                                Ok(_) => {
                                    tracing::info!("{} is logged out", local_login_session());
                                    let _ = socket
                                        .clone()
                                        .send(ClientEvent::Disconnect(local_login_session()))
                                        .await;
                                    // local storage for login
                                    *local_login_session.write() = disconnected_user();
                                }
                                Err(_) => tracing::info!("Error on {} logout", local_login_session()),
                            }
                        }
                        navigator.push(Route::Home {});
                    },
                    if snap_local_login_session == disconnected_user() {
                        "Sign in"
                    } else {
                        "Sign out"
                    }
                }
                Label { html_for: "navbar", "({snap_local_login_session})" }
            }
        }

        Outlet::<Route> {}
    }
}

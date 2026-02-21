use crate::{
    components::label::Label,
    websocket_handler::game_state::{GamePhase, ServerData},
    widgets::alert_dialog::AlertDialogComp,
};
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};

use crate::{
    auth_manager::server_fn::logout,
    common::{ADMIN, Route, disconnected_user},
    components::button::{Button, ButtonVariant},
    websocket_handler::event::{ClientEvent, ServerEvent},
};

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();

    // nav
    let navigator = use_navigator();

    // snapshot
    let snap_local_login_name_session = local_login_name_session();
    rsx! {
        div { class: "navbar",
            div { style: "display: flex; gap: 1rem;",
                Link { to: Route::Home {}, "Home" }
                if snap_local_login_name_session == ADMIN.to_string() {
                    Link { to: Route::AdminPage {}, "Admin" }
                }
            }
            div { style: "display: flex; flex-direction: row; gap: 1rem;",
                AlertDialogComp {}
                Button {
                    variant: if snap_local_login_name_session == disconnected_user() { ButtonVariant::Secondary } else { ButtonVariant::Destructive },
                    onclick: move |_| async move {
                        if local_login_name_session() != disconnected_user() {
                            match logout().await {
                                Ok(_) => {
                                    tracing::info!("{} is logged out", local_login_name_session());
                                    // notify server via websocket
                                    let _ = socket
                                        .clone()
                                        .send(ClientEvent::LogOut(local_login_name_session()))
                                        .await;
                                    // local storage for login
                                    *local_login_name_session.write() = disconnected_user();
                                    *local_login_id_session.write() = -1;
                                }
                                Err(_) => {
                                    tracing::info!("Error on {} logout", local_login_name_session())
                                }
                            }
                        }
                        navigator.push(Route::Home {});
                    },
                    if snap_local_login_name_session == disconnected_user() {
                        "Sign in"
                    } else {
                        "Sign out"
                    }
                }
                Label { html_for: "navbar", "({snap_local_login_name_session})" }
            }
        }

        Outlet::<Route> {}
    }
}

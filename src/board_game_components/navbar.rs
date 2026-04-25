use crate::{
    common::DISCONNECTED_USER,
    components::label::Label,
    websocket_handler::{NO_CLIENT_ID, msg_from_client::send_disconnect_from_server_data},
    widgets::alert_dialog::AlertDialogQuitGame,
};
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};

use crate::{
    auth_manager::server_fn::logout,
    common::{ADMIN, Route},
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
                Link {
                    to: Route::Home {},
                    onclick: move |_| async move {
                        send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                    },
                    "Home"
                }
                if snap_local_login_name_session == ADMIN.to_string() {
                    Link { to: Route::AdminPage {}, "Admin" }
                }
            }
            div { style: "display: flex; flex-direction: row; gap: 1rem;",
                AlertDialogQuitGame {}
                Button {
                    variant: if snap_local_login_name_session == *DISCONNECTED_USER { ButtonVariant::Secondary } else { ButtonVariant::Destructive },
                    onclick: move |_| async move {
                        if local_login_name_session() != *DISCONNECTED_USER {
                            match logout().await {
                                Ok(_) => {
                                    tracing::info!("{} is logged out", local_login_name_session());
                                    // notify server via websocket
                                    let _ = socket
                                        .clone()
                                        .send(ClientEvent::RequestLogOut(local_login_name_session()))
                                        .await;
                                    // local storage for login
                                    *local_login_name_session.write() = (*DISCONNECTED_USER).to_string();
                                    *local_login_id_session.write() = NO_CLIENT_ID;
                                }
                                Err(_) => {
                                    tracing::info!("Error on {} logout", local_login_name_session())
                                }
                            }
                        }
                        navigator.push(Route::Home {});
                    },
                    if snap_local_login_name_session == *DISCONNECTED_USER {
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

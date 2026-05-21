use crate::{
    common::DISCONNECTED_USER,
    websocket_handler::{NO_CLIENT_ID, msg_from_client::send_disconnect_from_server_data},
};
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::{
    auth_manager::server_fn::logout,
    common::{ADMIN, Route},
    components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
            AlertDialogRoot, AlertDialogTitle,
        },
        button::{Button, ButtonVariant},
    },
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::send_disconnect_from_server_data as send_quit,
    },
};

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    let server_data = use_context::<Signal<ServerData>>();

    // nav
    let navigator = use_navigator();

    // dialog open states — lifted here so the roots can live outside the navbar div
    let mut help_open = use_signal(|| false);
    let mut quit_open = use_signal(|| false);

    // snapshot
    let snap_local_login_name_session = local_login_name_session();

    rsx! {
        // ── Navbar bar ────────────────────────────────────────────────────────
        div { class: "navbar",
            // Left: brand + admin panel link
            div { style: "display: flex; align-items: center; gap: 1rem;",
                Link {
                    class: "navbar-brand",
                    to: Route::Home {},
                    onclick: move |_| async move {
                        send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                    },
                    "⚔️ RPG"
                }
                if snap_local_login_name_session == ADMIN.to_string() {
                    Link { class: "navbar-admin-link", to: Route::AdminPage {}, "🛡️ Panel" }
                }
            }
            // Right: trigger buttons only (no dialog roots here)
            div { style: "display: flex; flex-direction: row; align-items: center; gap: 0.75rem;",
                // Help trigger
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| help_open.set(true),
                    "?"
                }
                // Quit-game trigger (only while a game is running)
                if server_data().core_game_data.game_phase == GamePhase::Running {
                    Button {
                        onclick: move |_| quit_open.set(true),
                        r#type: "button",
                        "Quit game"
                    }
                }
                if snap_local_login_name_session != *DISCONNECTED_USER {
                    span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                }
                Button {
                    variant: if snap_local_login_name_session == *DISCONNECTED_USER { ButtonVariant::Secondary } else { ButtonVariant::Destructive },
                    onclick: move |_| async move {
                        if local_login_name_session() != *DISCONNECTED_USER {
                            match logout().await {
                                Ok(_) => {
                                    tracing::info!("{} is logged out", local_login_name_session());
                                    let _ = socket
                                        .clone()
                                        .send(ClientEvent::RequestLogOut(local_login_name_session()))
                                        .await;
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
            }
        }

        // ── Dialog roots — rendered at layout level, NOT inside the navbar div ──

        // Help dialog
        AlertDialogRoot { open: help_open(), on_open_change: move |v| help_open.set(v),
            AlertDialogContent {
                AlertDialogTitle { "How to play" }
                AlertDialogDescription {
                    div { style: "text-align:left; line-height:1.7;",
                        p { "1. 🔐 Sign in or register on the login page." }
                        p { "2. 🎮 Create a new game or join an ongoing one from the home page." }
                        p {
                            "3. 🧙 In the Lobby, select your character with the dropdown. Wait for all players to pick one."
                        }
                        p {
                            "4. ▶️ The host (server owner) clicks 'Start Game' when everyone is ready."
                        }
                        p {
                            "5. ⚔️ On your turn, click the ⚔️ button on your character card to open the attack list, then click an attack."
                        }
                        p {
                            "6. 🎯 Click the target buttons that appear on characters to set your target(s), then confirm with '⚔️ Launch Attack'."
                        }
                        p { "7. 💊 Click the 💊 button on your character card to use a potion." }
                        p { "8. 📦 Open 'Inventory' (game toolbar) to see your stats and equipment." }
                        p { "9. 📊 Open 'Game Stats' (game toolbar) to track damage and healing." }
                        p {
                            "10. 🏆 At end of scenario the host loads the next one; at end of game, replay or quit."
                        }
                    }
                }
                AlertDialogAction {
                    AlertDialogCancel { "Close" }
                }
            }
        }

        // Quit-game confirmation dialog
        AlertDialogRoot { open: quit_open(), on_open_change: move |v| quit_open.set(v),
            AlertDialogContent {
                AlertDialogTitle { "Quit Game" }
                AlertDialogDescription { "Are you sure you want to quit the game?" }
                AlertDialogAction {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction {
                        on_click: move |_| {
                            async move {
                                send_quit(socket, &local_login_name_session()).await;
                                let navigator = use_navigator();
                                navigator.push(Route::Home {});
                            }
                        },
                        "Confirm"
                    }
                }
            }
        }

        Outlet::<Route> {}

        // ── Footer ────────────────────────────────────────────────────────────
        footer { class: "app-footer",
            div { class: "app-footer-inner",
                div { class: "app-footer-brand",
                    span { class: "app-footer-icon", "⚔️" }
                    span { class: "app-footer-name", "dx-rpg" }
                    span { class: "app-footer-version", {concat!("v", env!("CARGO_PKG_VERSION"))} }
                }
                div { class: "app-footer-info",
                    span { "Built with " }
                    a {
                        href: "https://dioxuslabs.com",
                        target: "_blank",
                        rel: "noopener",
                        "Dioxus"
                    }
                    span { " · " }
                    span { "⚡ Rust + WASM" }
                }
            }
        }
    }
}

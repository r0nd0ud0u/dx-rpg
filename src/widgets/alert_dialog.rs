use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_i18n::t;
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::{
    common::Route,
    components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
            AlertDialogRoot, AlertDialogTitle,
        },
        button::Button,
    },
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::send_disconnect_from_server_data,
    },
};

// NOTE: not currently referenced anywhere — navbar.rs has its own, wired-up
// quit-game dialog. Kept in sync with the same t!() keys in case this gets
// reconnected; a future cleanup pass may want to just delete it.
#[component]
pub fn AlertDialogQuitGame() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let server_data = use_context::<Signal<ServerData>>();
    // local signal
    let mut open = use_signal(|| false);
    rsx! {
        if server_data().core_game_data.game_phase == GamePhase::Running {
            Button {
                onclick: move |_| open.set(true),
                r#type: "button",
                {t!("navbar-quit-game")}
            }
        }
        AlertDialogRoot { open: open(), on_open_change: move |v| open.set(v),
            AlertDialogContent {
                // You may pass class/style for custom appearance
                AlertDialogTitle { {t!("quit-dialog-title")} }
                AlertDialogDescription { {t!("quit-dialog-body")} }
                AlertDialogAction {
                    AlertDialogCancel { {t!("common-cancel")} }
                    AlertDialogAction {
                        on_click: move |_| {
                            async move {
                                send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                                let navigator = use_navigator();
                                navigator.push(Route::Home {});
                            }
                        },
                        {t!("common-confirm")}
                    }
                }
            }
        }
    }
}

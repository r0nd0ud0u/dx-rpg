use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
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

#[component]
pub fn AlertDialogComp() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let server_data = use_context::<Signal<ServerData>>();
    // local signal
    let mut open = use_signal(|| false);
    rsx! {
        if server_data().core_game_data.game_phase == GamePhase::Running {
            Button { onclick: move |_| open.set(true), r#type: "button", "Quit game" }
        }
        AlertDialogRoot { open: open(), on_open_change: move |v| open.set(v),
            AlertDialogContent {
                // You may pass class/style for custom appearance
                AlertDialogTitle { "Quit Game" }
                AlertDialogDescription { "Are you sure to quit the game ?" }
                AlertDialogAction {
                    AlertDialogCancel { "Cancel" }
                    AlertDialogAction {
                        on_click: move |_| {
                            async move {
                                send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                                let navigator = use_navigator();
                                navigator.push(Route::Home {});
                            }
                        },
                        "Confirm"
                    }
                }
            }
        }
    }
}

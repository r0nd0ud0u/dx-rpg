use dioxus::prelude::*;
use dioxus_fullstack::{WebSocketOptions, use_websocket};

use crate::websocket_components::{
    event::{ClientEvent, ServerEvent, new_event},
    game_state::GameStateWebsocket,
    player_name::{PlayerName, random_name},
};

#[allow(clippy::redundant_closure)]
pub static PLAYER_NAME: GlobalSignal<String> = Signal::global(|| String::new());

/// Tracks whether a name change is pending confirmation from the server.
/// This is provided at the components (crate) level so both the app-level
/// socket loop and the `PlayerName` component can read/update it.
pub static PLAYER_NAME_PENDING: GlobalSignal<bool> = Signal::global(|| false);

pub fn WebsocketApp() -> Element {
    // Local UI state
    let mut message = use_signal(String::new);
    let mut player_id = use_signal(|| 0);
    let mut game_state = use_signal(GameStateWebsocket::default);

    if PLAYER_NAME.read().trim().is_empty() {
        *PLAYER_NAME.write() = random_name();
    }

    // Open the websocket without sending the name in a query param.
    // The client will send an explicit `ClientEvent::SetName` after connect
    // to initialize the player's name on the server.
    let socket = use_websocket(|| new_event(WebSocketOptions::new()));

    // Receive events from the websocket and update local signals.
    // Also send an initial `SetName` to the server once the socket is created.
    use_future(move || {
        let mut socket = socket;
        // Capture the initial name at mount time
        let initial_name = PLAYER_NAME.read().to_string();
        async move {
            // Mark that we're awaiting confirmation for the initial name and send it.
            *PLAYER_NAME_PENDING.write() = true;
            let _ = socket.send(ClientEvent::SetName(initial_name)).await;

            while let Ok(event) = socket.recv().await {
                match event {
                    ServerEvent::Message(msg) => {
                        message.set(msg);
                    }
                    ServerEvent::AssignPlayerId(id) => {
                        player_id.set(id);
                    }
                    ServerEvent::NameAccepted(accepted_name) => {
                        // Server confirmed the name change (initial or rename).
                        // Update the global player name and clear the pending flag.
                        *PLAYER_NAME.write() = accepted_name.clone();
                        *PLAYER_NAME_PENDING.write() = false;
                    }
                    ServerEvent::SnapshotPlayers(gs) => {
                        game_state.set(gs);
                    }
                }
            }
        }
    });

    let socket_status = format!("{:?}", socket.status());
    let current_message = message();
    let current_gs = game_state();

    use_context_provider(|| socket);
    use_context_provider(|| player_id);

    rsx! {
        div {

            h2 { "Test websocket" }
            p {
                small { "Connection status: {socket_status}" }
            }
            p { "Message: {current_message}" }

            PlayerName {}
            for s in current_gs.players {
                p { "{s}" }
            }
        }
    }
}

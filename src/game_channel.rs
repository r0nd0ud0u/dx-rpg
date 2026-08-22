//! Abstracts "send a `ClientEvent`, receive `ServerEvent`s back" over either a real
//! websocket (remote or local server, unchanged from before this module existed) or —
//! client builds only — a direct in-process call into `local_engine` for offline mode
//! (see `local_channel.rs`). The ~17 files that just do
//! `socket.send(ClientEvent::X(...)).await` don't need to know or care which backend is
//! active; they only need `use_context::<UseWebsocket<ClientEvent, ServerEvent,
//! CborEncoding>>()` swapped for `use_context::<GameChannel>()`.
//!
//! `GameChannel` always holds the real websocket handle too, unconditionally — the
//! server build (which also renders this same UI for SSR) only ever uses that variant;
//! there's no "offline mode" concept server-side, so `Local`/`offline` are entirely
//! absent from that build rather than merely unused.
//!
//! `local` is wrapped in `CopyValue` (not held directly) so `GameChannel` itself stays
//! `Copy`, matching `UseWebsocket`'s ergonomics exactly — every one of the ~17 call
//! sites already does `onclick: move |_| async move { socket.send(...).await }`, which
//! needs `socket` to be freely re-capturable on every click (an `FnMut` closure moving a
//! non-`Copy` value into its own inner `async move` block only implements `FnOnce`,
//! i.e. "usable for exactly one click"). `LocalChannel` itself deliberately stays
//! `Rc`/`RefCell`-based, not `CopyValue`-based — `CopyValue::new` requires an active
//! Dioxus component scope, which would break `local_channel.rs`'s own unit tests (they
//! construct a `LocalChannel` directly, no `VirtualDom` running at all).

use dioxus::fullstack::{CborEncoding, UseWebsocket};
#[cfg(not(feature = "server"))]
use dioxus::prelude::{CopyValue, ReadableExt, WritableExt};

use crate::websocket_handler::event::{ClientEvent, ServerEvent};

#[derive(Debug)]
pub enum GameChannelError {
    // Boxed: WebsocketError's largest variant is 136+ bytes, which would otherwise
    // make every `Result<_, GameChannelError>` that size even on the success path.
    Remote(Box<dioxus::fullstack::WebsocketError>),
    /// The local channel's receiver was dropped — shouldn't happen during a live
    /// offline session (its sender lives as long as the GameChannel that owns it).
    #[cfg(not(feature = "server"))]
    LocalClosed,
}

#[derive(Clone, Copy)]
pub struct GameChannel {
    remote: UseWebsocket<ClientEvent, ServerEvent, CborEncoding>,
    #[cfg(not(feature = "server"))]
    local: CopyValue<crate::local_channel::LocalChannel>,
    /// Which backend `send`/`recv` actually use. A `Signal` (not a plain bool) so
    /// flipping it from the Home page's "Play Offline" button is immediately visible
    /// to every clone of this `GameChannel` already handed out via context — same
    /// reason the rest of this codebase threads settings through `Signal`-wrapped
    /// context newtypes rather than plain values.
    #[cfg(not(feature = "server"))]
    offline: dioxus::prelude::Signal<bool>,
}

impl GameChannel {
    pub fn new(
        remote: UseWebsocket<ClientEvent, ServerEvent, CborEncoding>,
        #[cfg(not(feature = "server"))] local: crate::local_channel::LocalChannel,
        #[cfg(not(feature = "server"))] offline: dioxus::prelude::Signal<bool>,
    ) -> Self {
        Self {
            remote,
            #[cfg(not(feature = "server"))]
            local: CopyValue::new(local),
            #[cfg(not(feature = "server"))]
            offline,
        }
    }

    /// Never called on the server build — `send`/`recv` only reference this from
    /// inside a `#[cfg(not(feature = "server"))]` block, since the server has no
    /// concept of offline mode at all (see this module's doc comment).
    #[cfg(not(feature = "server"))]
    fn is_offline(&self) -> bool {
        (self.offline)()
    }

    /// Starts an offline session (see `LocalChannel::activate`) and flips the routing
    /// flag so every existing `.send()`/`.recv()` call site transparently switches to
    /// it — call once, from the Home page's "Play Offline" action.
    #[cfg(not(feature = "server"))]
    pub fn go_offline(&mut self) {
        self.local.read().activate();
        self.offline.set(true);
    }

    pub async fn send(&self, msg: ClientEvent) -> Result<(), GameChannelError> {
        #[cfg(not(feature = "server"))]
        if self.is_offline() {
            self.local.read().send(msg);
            return Ok(());
        }
        self.remote
            .send(msg)
            .await
            .map_err(|e| GameChannelError::Remote(Box::new(e)))
    }

    pub async fn recv(&mut self) -> Result<ServerEvent, GameChannelError> {
        #[cfg(not(feature = "server"))]
        if self.is_offline() {
            // Cloned out of the CopyValue's temporary read guard (cheap — LocalChannel
            // is just a bundle of Rc clones) before awaiting: the guard itself can't
            // live across the .await (it's a short-lived borrow), but the owned
            // LocalChannel clone can.
            let local = self.local.read().clone();
            return local.recv().await.ok_or(GameChannelError::LocalClosed);
        }
        self.remote
            .recv()
            .await
            .map_err(|e| GameChannelError::Remote(Box::new(e)))
    }
}

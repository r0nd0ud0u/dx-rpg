//! Abstracts "send a `ClientEvent`, receive `ServerEvent`s back" over either a real
//! websocket or — client builds only — a direct in-process call into `local_engine` for
//! offline mode (see `local_channel.rs`). Call sites just `socket.send(...).await` and
//! don't know which backend is active.
//!
//! The websocket handle is always present; `Local`/`offline` are cfg'd out of the server
//! build entirely, which has no notion of offline mode.
//!
//! `local` is wrapped in `CopyValue` so `GameChannel` stays `Copy` like `UseWebsocket`:
//! call sites do `move |_| async move { socket.send(...) }`, and an `FnMut` closure that
//! moves a non-`Copy` value into an inner `async move` is only `FnOnce` — usable for one
//! click. `LocalChannel` itself stays `Rc`/`RefCell`: `CopyValue::new` needs a live
//! component scope, which its unit tests don't have.

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
    /// Which backend `send`/`recv` use. A `Signal` so flipping it is visible to every
    /// `GameChannel` clone already handed out via context.
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

    /// Offline (single-player) mode: picks the backend, and hides Navbar's connection
    /// badge. Always `false` on the server build.
    #[cfg(not(feature = "server"))]
    pub fn is_offline(&self) -> bool {
        (self.offline)()
    }

    #[cfg(feature = "server")]
    pub fn is_offline(&self) -> bool {
        false
    }

    /// Starts an offline session (`LocalChannel::activate`) and switches every call site
    /// over to it.
    #[cfg(not(feature = "server"))]
    pub fn go_offline(&mut self) {
        self.local.read().activate();
        self.offline.set(true);
    }

    /// Back to the real socket. Sign-out calls this, or the login page could never reach
    /// a server again without an app restart.
    #[cfg(not(feature = "server"))]
    pub fn go_online(&mut self) {
        self.offline.set(false);
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
        {
            // Cloned out of the read guard before awaiting — the guard can't cross an
            // `.await`, the clone (a bundle of Rcs) can.
            let local = self.local.read().clone();
            if self.is_offline() {
                // Never poll `remote` here: nothing listens on SERVER_URL offline, and if
                // it resolves immediately racing it would busy-spin.
                return local.recv().await.ok_or(GameChannelError::LocalClosed);
            }
            // Not offline yet, but `go_offline()` can fire while this `remote.recv()` is
            // in flight — which may never resolve on its own. Race both; `local` wins
            // ties and stays pending until `go_offline()` has run.
            let local_fut = local.recv();
            let remote_fut = self.remote.recv();
            futures::pin_mut!(local_fut);
            futures::pin_mut!(remote_fut);
            return match futures::future::select(local_fut, remote_fut).await {
                futures::future::Either::Left((res, _)) => res.ok_or(GameChannelError::LocalClosed),
                futures::future::Either::Right((res, _)) => {
                    res.map_err(|e| GameChannelError::Remote(Box::new(e)))
                }
            };
        }
        #[cfg(feature = "server")]
        self.remote
            .recv()
            .await
            .map_err(|e| GameChannelError::Remote(Box::new(e)))
    }
}

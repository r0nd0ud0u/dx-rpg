//! In-app log capture, so admins can read `tracing` output on platforms with no
//! attached developer console (namely mobile — there's no easy way to see stdout/logcat
//! output from an installed app without a cable and a second machine).
//!
//! This installs an extra `tracing_subscriber` layer alongside the normal
//! stdout/wasm-console output, mirroring every event into an in-memory ring buffer that
//! [`crate::board_game_components::debug_console::DebugConsole`] renders.

use std::collections::VecDeque;
use std::sync::Mutex;

use dioxus::logger::tracing::{
    self, Event, Level, Subscriber,
    field::{Field, Visit},
};
use once_cell::sync::Lazy;
use tracing_subscriber::{
    Layer, Registry,
    layer::{Context, SubscriberExt},
};

/// Oldest lines are evicted once the buffer grows past this, so a long-running session
/// can't leak memory just from logging.
const MAX_LOG_LINES: usize = 500;

static LOG_BUFFER: Lazy<Mutex<VecDeque<String>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)));

/// Snapshot of the captured log lines, oldest first.
pub fn recent_logs() -> Vec<String> {
    LOG_BUFFER.lock().unwrap().iter().cloned().collect()
}

pub fn clear_logs() {
    LOG_BUFFER.lock().unwrap().clear();
}

#[derive(Default)]
struct MessageVisitor(String);

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}

struct InAppLogLayer;

impl<S: Subscriber> Layer<S> for InAppLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let line = format!(
            "[{}] {}: {}",
            event.metadata().level(),
            event.metadata().target(),
            visitor.0
        );

        let mut buffer = LOG_BUFFER.lock().unwrap();
        if buffer.len() >= MAX_LOG_LINES {
            buffer.pop_front();
        }
        buffer.push_back(line);
    }
}

/// Replaces the plain `dioxus::logger::init(level)` call in `main()`. Sets up the same
/// per-platform output (wasm console on web, stdout elsewhere) `dioxus-logger` would,
/// plus [`InAppLogLayer`] on every platform so logs are always readable from within the
/// app itself, not just from an attached console.
pub fn init(level: Level) {
    #[cfg(target_arch = "wasm32")]
    {
        let layer_config = tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(level)
            .build();
        let wasm_layer = tracing_wasm::WASMLayer::new(layer_config);
        let _ = tracing::subscriber::set_global_default(
            Registry::default().with(wasm_layer).with(InAppLogLayer),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let filter = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(level.into())
            .from_env_lossy()
            .add_directive("hyper_util=warn".parse().unwrap());
        let _ = tracing::subscriber::set_global_default(
            Registry::default()
                .with(filter)
                .with(tracing_subscriber::fmt::layer())
                .with(InAppLogLayer),
        );
    }
}

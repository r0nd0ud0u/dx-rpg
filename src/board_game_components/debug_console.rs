use std::time::Duration;

use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::{
    components::{
        alert_dialog::{
            AlertDialogActions, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
            AlertDialogRoot, AlertDialogTitle,
        },
        button::{Button, ButtonVariant},
    },
    debug_console::{clear_logs, recent_logs},
};

/// How often the log view refreshes while open. Captured logs live in a plain
/// `Mutex<VecDeque<_>>` (see src/debug_console.rs) rather than a reactive Signal, since
/// tracing events fire from arbitrary tasks/threads — polling while the panel is open is
/// simplest given how rarely an admin has it open.
const REFRESH_INTERVAL: Duration = Duration::from_millis(1000);

/// Admin-only in-app log viewer, so logs are readable on platforms with no attached
/// developer console (mobile, mainly — there's no easy way to see stdout/logcat output
/// from an installed app without a cable and a second machine).
#[component]
pub fn DebugConsole(open: bool, on_open_change: EventHandler<bool>) -> Element {
    // `open` is a plain prop, not a Signal, so a raw read of it inside use_effect below
    // wouldn't register as a tracked dependency and the effect would never rerun after
    // mount. Mirroring it into a Signal here (updated on every render, which happens
    // whenever the parent passes a new value) gives the effect something reactive to
    // watch for the open/close transition.
    let mut open_signal = use_signal(|| open);
    if open_signal() != open {
        open_signal.set(open);
    }

    let mut logs = use_signal(recent_logs);

    use_effect(move || {
        if !open_signal() {
            return;
        }
        spawn(async move {
            loop {
                if !open_signal() {
                    break;
                }
                logs.set(recent_logs());
                dioxus_sdk_time::sleep(REFRESH_INTERVAL).await;
            }
        });
    });

    rsx! {
        AlertDialogRoot { open, on_open_change: move |v| on_open_change.call(v),
            AlertDialogContent {
                AlertDialogTitle { {t!("debug-console-title")} }
                AlertDialogDescription {
                    div { style: "text-align:left; max-height:60vh; overflow-y:auto; padding-right:4px;",
                        if logs().is_empty() {
                            p { {t!("debug-console-empty")} }
                        } else {
                            pre { style: "white-space:pre-wrap; word-break:break-word; font-size:0.8em; margin:0;",
                                {logs().join("\n")}
                            }
                        }
                    }
                }
                AlertDialogActions {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            clear_logs();
                            logs.set(Vec::new());
                        },
                        {t!("debug-console-clear")}
                    }
                    AlertDialogCancel { {t!("common-close")} }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::components::{
    button::Button,
    popover::{PopoverContent, PopoverRoot, PopoverTrigger},
};

// Unreferenced anywhere in the app (no route/page renders it) — kept translated for consistency.
#[component]
pub fn PopoverComp() -> Element {
    let mut open = use_signal(|| false);
    let mut confirmed = use_signal(|| false);

    rsx! {
        PopoverRoot { open: open(), on_open_change: move |v| open.set(v),
            PopoverTrigger { {t!("popover-demo-trigger")} }
            PopoverContent { gap: "0.25rem",
                h3 {
                    padding_top: "0.25rem",
                    padding_bottom: "0.25rem",
                    width: "100%",
                    text_align: "center",
                    margin: 0,
                    {t!("popover-demo-title")}
                }
                Button {
                    r#type: "button",
                    "data-style": "outline",
                    onclick: move |_| {
                        open.set(false);
                        confirmed.set(true);
                    },
                    {t!("common-confirm")}
                }
                Button {
                    r#type: "button",
                    "data-style": "outline",
                    onclick: move |_| {
                        open.set(false);
                    },
                    {t!("common-cancel")}
                }
            }
        }
        if confirmed() {
            p { style: "color: var(--contrast-error-color); margin-top: 16px; font-weight: 600;",
                {t!("popover-demo-confirmed")}
            }
        }
    }
}

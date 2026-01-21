use dioxus::prelude::*;
use dioxus_primitives::label::Label;

use crate::common::{Route, USER_NAME};

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { class: "navbar",
            Link { to: Route::Home {}, "Home" }
            Label { html_for: "navbar", "{USER_NAME.read()}" }
        }

        Outlet::<Route> {}
    }
}

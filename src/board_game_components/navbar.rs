use dioxus::prelude::*;

use crate::common::Route;

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { class: "navbar",
            Link { to: Route::Home {}, "Home" }
        }

        Outlet::<Route> {}
    }
}

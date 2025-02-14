use dioxus::prelude::*;

#[component]
pub fn Character_page(name: String) -> Element {
    let mut vie = use_signal(|| 100);
    rsx! {
        div { class: "container-bar",
            div {
                class: "life-bar",
                width: "{vie}%",
                background_color: get_color(vie()),
            }
        }

        button { class: "damages-btn", onclick: move |_| vie -= 20, "Give damages" }
    }
}

fn get_color(vie: i32) -> &'static str {
    if vie > 80 {
        "green"
    } else if vie > 20 {
        "orange"
    } else {
        "red"
    }
}

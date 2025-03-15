use dioxus::prelude::*;

#[component]
pub fn Character_page(name: String) -> Element {
    let max_life = 100;
    let mut life = use_signal(|| max_life);
    rsx! {
        div { id: "character",
            h4 { {name} }
            div { class: "container-bar",
                div {
                    class: "life-bar",
                    width: "{life}%",
                    background_color: get_color(life()),
                }
                span { class: "bar-text", "{life()} / {max_life}" }
            }
        }

        button {
            class: "damages-btn",
            onclick: move |_| life.set((life() - 10).max(0)),
            "Give damages"
        }
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

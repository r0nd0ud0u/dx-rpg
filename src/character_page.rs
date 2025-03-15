use dioxus::prelude::*;
use lib_rpg::character::Character;

#[component]
pub fn Character_page(c: Character) -> Element {
    let max_life = c.stats.hp.max;
    let mut life = use_signal(|| c.stats.hp.current);
    rsx! {
        div { id: "character",
            h4 { {c.name} }
            div { class: "container-bar",
                div {
                    class: "life-bar",
                    width: "{life}%",
                    background_color: get_color(life() as i32),
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

fn get_color(life: i32) -> &'static str {
    if life > 80 {
        "green"
    } else if life > 20 {
        "orange"
    } else {
        "red"
    }
}

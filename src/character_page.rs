use dioxus::prelude::*;
use lib_rpg::character::Character;

pub const PATH_IMG: &str = "assets/img";

#[component]
pub fn CharacterPanel(c: Character) -> Element {
    rsx! {
        div { class: "character",
            img {
                src: format!("{}/{}.png", PATH_IMG, "Troll"),
                class: "image-small",
            }
            h4 { {c.name} }
            div {
                if c.stats.hp.max > 0 {
                    BarComponent {
                        max: c.stats.hp.max,
                        current: c.stats.hp.current,
                        name: "HP",
                    }
                }
                if c.stats.mana.max > 0 {
                    BarComponent {
                        max: c.stats.mana.max,
                        current: c.stats.mana.current,
                        name: "MP",
                    }
                }
                if c.stats.vigor.max > 0 {
                    BarComponent {
                        max: c.stats.vigor.max,
                        current: c.stats.vigor.current,
                        name: "VP",
                    }
                }
                if c.stats.berseck.max > 0 {
                    BarComponent {
                        max: c.stats.berseck.max,
                        current: c.stats.berseck.current,
                        name: "BP",
                    }
                }
            }
        }
    }
}

#[component]
pub fn BarComponent(max: u32, current: u32, name: String) -> Element {
    let mut current_sig = use_signal(|| current);
    rsx! {
        div { class: "grid-container",
            h4 { {name} }
            div { class: "container-bar",
                div {
                    class: "life-bar",
                    width: "{current_sig}%",
                    background_color: get_color(current_sig() as i32),
                }
            }
            h4 { "{current_sig()} / {max}" }
        }
    }
}

fn get_color(value: i32) -> &'static str {
    if value > 80 {
        "green"
    } else if value > 20 {
        "orange"
    } else {
        "red"
    }
}

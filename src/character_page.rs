use dioxus::prelude::*;
use lib_rpg::{
    character::{Character, CharacterType},
    common::stats_const::*,
};

pub const PATH_IMG: &str = "assets/img";

#[component]
pub fn CharacterPanel(c: Character) -> Element {
    let bg = if c.kind == CharacterType::Hero {
        "blue"
    } else {
        "red"
    };
    rsx! {
        div { class: "character", background_color: bg,
            div {
                img {
                    src: format!("{}/{}.png", PATH_IMG, c.photo_name.clone()),
                    class: "image-small",
                }
                h4 { {c.short_name.clone()} }
            }
            div {
                if c.stats.all_stats[HP].max > 0 {
                    BarComponent {
                        max: c.stats.all_stats[HP].max,
                        current: c.stats.all_stats[HP].current,
                        name: "HP",
                    }
                }
                if c.stats.all_stats[MANA].max > 0 {
                    BarComponent {
                        max: c.stats.all_stats[MANA].max,
                        current: c.stats.all_stats[MANA].current,
                        name: "MP",
                    }
                }
                if c.stats.all_stats[VIGOR].max > 0 {
                    BarComponent {
                        max: c.stats.all_stats[VIGOR].max,
                        current: c.stats.all_stats[VIGOR].current,
                        name: "VP",
                    }
                }
                if c.stats.all_stats[BERSECK].max > 0 {
                    BarComponent {
                        max: c.stats.all_stats[BERSECK].max,
                        current: c.stats.all_stats[BERSECK].current,
                        name: "BP",
                    }
                }
            }
        }
    }
}

#[component]
pub fn BarComponent(max: u64, current: u64, name: String) -> Element {
    let width_display = current*100/ max;
    rsx! {
        div { class: "grid-container",
            h4 { {name} }
            div { class: "container-bar",
                div {
                    class: "life-bar",
                    width: "{width_display}%",
                    background_color: get_color(width_display as i32),
                }
            }
            h4 { "{current} / {max}" }
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

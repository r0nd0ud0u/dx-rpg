use dioxus::prelude::*;
use lib_rpg::{
    character::{Character, CharacterType},
    common::stats_const::*, testing_target,
};

use crate::common::APP;

pub const PATH_IMG: &str = "assets/img";

#[component]
pub fn CharacterPanel(c: Character, is_current_player: bool) -> Element {
    let mut atk_menu_display = use_signal(|| false);
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
        if is_current_player {
            if c.kind == CharacterType::Hero {
                button { class: "menu-atk-button", onclick: move |_| async move {
                    atk_menu_display.set(!atk_menu_display());
                }, "ATK" }
            } else if c.kind == CharacterType::Boss {   
                button {
                    class: "atk-button-ennemy",
                    onclick: move |_| async move {},
                    "ATK On Going"
                }
            }
            if atk_menu_display() {
                AttackList { c: c.clone(), display_atklist_sig: atk_menu_display }
            }
        }
        button {
            class: "character-name-button",
            background_color: "black",
            onclick: move |_| async move {},
            "{c.name}"
        }
    }
}

#[component]
pub fn BarComponent(max: u64, current: u64, name: String) -> Element {
    let width_display = current * 100 / max;
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

#[component]
pub fn AttackList(c: Character, display_atklist_sig: Signal<bool>) -> Element {
    rsx! {
        div {   
            class: "attack-list",
            for (key, value) in c.attacks_list.iter() {
                button {
                    class: "atk-button",
                    background_color: "black",
                    onclick: move |_| async move {
                        *display_atklist_sig.write() = false;
                        APP.write()
                        .game_manager
                        .launch_attack(
                            "SimpleAtk",
                            vec![testing_target::build_target_angmar_indiv()],
                        );
                    },
                    "{key}"
                }
            }
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

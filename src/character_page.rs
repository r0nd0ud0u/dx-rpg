use std::{collections::HashMap, time::Duration};

use dioxus::prelude::*;
use indexmap::IndexMap;
use lib_rpg::{
    character::{Character, CharacterType},
    common::stats_const::*,
    testing_target,
};

use crate::common::APP;

pub const PATH_IMG: &str = "assets/img";

#[component]
pub fn CharacterPanel(c: Character, is_auto_atk: bool) -> Element {
    let mut atk_menu_display = use_signal(|| false);
    let bg = if c.kind == CharacterType::Hero {
        "blue"
    } else {
        "red"
    };
    let energy_list = IndexMap::from([
        (HP.to_owned(), HP.to_owned()),
        (MANA.to_owned(), "MP".to_owned()),
        (VIGOR.to_owned(), "VP".to_owned()),
        (BERSECK.to_owned(), "BP".to_owned()),
    ]);
    use_resource(use_reactive!(|(is_auto_atk,)| async move {
        // Simulate a delay before launching the attack
        // use wasmtimer instead of tokio::time to make it work with wasm
        // We manually add the resource to the dependencies list with the `use_reactive` hook
        // Any time `is_auto_atk` changes, the resource will rerun
        if is_auto_atk {
            wasmtimer::tokio::sleep(Duration::from_millis(1000)).await;
            APP.write().game_manager.launch_attack(
                "SimpleAtk",
                vec![testing_target::build_target_angmar_indiv()],
            )
        }
    }));

    rsx! {
        div { class: "character", background_color: bg,
            div {
                img {
                    src: format!("{}/{}.png", PATH_IMG, c.photo_name.clone()),
                    class: "image-small",
                }
            }
            div {
                for (stat , display_stat) in energy_list.iter() {
                    if c.stats.all_stats[stat].max > 0 {
                        BarComponent {
                            max: c.stats.all_stats[stat].max,
                            current: c.stats.all_stats[stat].current,
                            name: display_stat,
                        }
                    }
                }
            }
        }
        if is_auto_atk {
            button { class: "atk-button-ennemy", onclick: move |_| async move {}, "ATK On Going" }
        } else if c.kind == CharacterType::Hero {
            button {
                class: "menu-atk-button",
                onclick: move |_| async move {
                    atk_menu_display.set(!atk_menu_display());
                },
                "ATK"
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
        div { class: "attack-list",
            for (key , value) in c.attacks_list.iter() {
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

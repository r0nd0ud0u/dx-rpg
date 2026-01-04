use colorgrad::Gradient;
use dioxus::prelude::*;
use indexmap::IndexMap;
use lib_rpg::{
    attack_type::AttackType,
    character::{Character, CharacterType},
    common::stats_const::*,
};

use crate::common::PATH_IMG;
use crate::common::{APP, ENERGY_GRAD};

#[component]
pub fn CharacterPanel(
    c: Character,
    current_player_name: String,
    selected_atk: Signal<AttackType>,
    atk_menu_display: Signal<bool>,
    write_game_manager: Signal<bool>,
    is_auto_atk: ReadSignal<bool>,
) -> Element {
    // if boss is dead, panel is hidden
    if c.is_dead().is_some_and(|value| value) && c.kind == CharacterType::Boss {
        return rsx! {};
    }
    let bg = if c.kind == CharacterType::Hero {
        "blue"
    } else {
        "red"
    };
    let energy_list = IndexMap::from([
        (HP.to_owned(), HP.to_owned()),
        (MANA.to_owned(), "MP".to_owned()),
        (VIGOR.to_owned(), "VP".to_owned()),
        (BERSERK.to_owned(), "BP".to_owned()),
    ]);

    let name2 = c.name.clone();
    let kind2 = c.kind.clone();
    rsx! {
        div { class: "character", background_color: bg,
            div {
                img {
                    src: format!("{}/{}.png", PATH_IMG, c.photo_name),
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
                h4 { "Lvl: {c.level}" }
            }
        }
        // atk button
        if is_auto_atk() {
            button { class: "atk-button-ennemy", onclick: move |_| async move {}, "ATK On Going" }
        } else if kind2.clone() == CharacterType::Hero && current_player_name == name2.clone() {
            button {
                class: "menu-atk-button",
                onclick: move |_| async move {
                    atk_menu_display.set(!atk_menu_display());
                },
                "ATK"
            }
        }
        // name button
        button {
            class: "character-name-button",
            background_color: "black",
            onclick: move |_| async move {},
            "{name2.clone()}"
        }
        // target button
        if !selected_atk().name.is_empty() {
            CharacterTargetButton { c: c.clone(), selected_atk, write_game_manager }
        }
    }
}

#[component]
pub fn CharacterTargetButton(
    c: Character,
    selected_atk: Signal<AttackType>,
    write_game_manager: Signal<bool>,
) -> Element {
    let mut kind_str = "hero";
    if c.kind == CharacterType::Boss {
        kind_str = "boss";
    }
    rsx! {
        if c.is_current_target {
            button {
                class: format!("{}-target-button-active", kind_str),
                onclick: move |_| async move {},
                ""
            }
        } else if c.is_potential_target {
            button {
                class: format!("{}-target-button", kind_str),
                onclick: move |_| {
                    let new_target_name = c.name.clone();
                    async move {
                        APP.write()
                            .game_manager
                            .pm
                            .set_one_target(&new_target_name, &selected_atk().reach);
                        write_game_manager.set(true);
                    }
                },
                ""
            }
        }
    }
}

#[component]
pub fn BarComponent(max: u64, current: u64, name: String) -> Element {
    let width_display = current * 100 / max;
    rsx! {
        div { class: "grid-container",
            div { class: "text-bar", {name} }
            div { class: "container-bar",
                div {
                    class: "life-bar",
                    width: "{width_display}%",
                    background_color: get_color(width_display as i32),
                }
            }
            div { class: "text-bar", "{current} / {max}" }
        }
    }
}

#[component]
pub fn NewAtkButton(
    attack_type: AttackType,
    display_atklist_sig: Signal<bool>,
    selected_atk: Signal<AttackType>,
    launcher: Character,
    write_game_manager: Signal<bool>,
) -> Element {
    let can_be_launched = launcher.can_be_launched(&attack_type);
    let mut color = "grey";
    if !can_be_launched {
        color = "black";
    }
    rsx! {
        button {
            class: "atk-button",
            background_color: color,
            onclick: move |_| {
                let value = attack_type.clone();
                let l_launcher = launcher.clone();
                async move {
                    APP.write().game_manager.pm.set_targeted_characters(&l_launcher, &value);
                    *display_atklist_sig.write() = false;
                    selected_atk.set(value);
                    write_game_manager.set(true);
                }
            },
            disabled: !can_be_launched,
            "{attack_type.name}"
        }
    }
}

#[component]
pub fn AttackList(
    name: String,
    display_atklist_sig: Signal<bool>,
    selected_atk: Signal<AttackType>,
    write_game_manager: Signal<bool>,
) -> Element {
    if let Some(c) = APP.read().game_manager.pm.get_active_character(&name) {
        rsx! {
            div { class: "attack-list",
                for (_key , value) in c.attacks_list.iter() {
                    if c.level >= value.level as u64 {
                        div { class: "attack-list-line",
                            button {
                                class: "atk-type-button",
                                background_color: get_type_color(value),
                                onclick: move |_| {},
                                ""
                            }
                            NewAtkButton {
                                attack_type: value.clone(),
                                display_atklist_sig,
                                selected_atk,
                                launcher: c.clone(),
                                write_game_manager,
                            }
                            button {
                                class: "cost-energy-button",
                                onclick: move |_| {},
                                {get_cost(value)}
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn get_color(value: i32) -> String {
    ENERGY_GRAD.at(value as f32 / 100.0).to_css_hex()
}

fn get_type_color(atk: &AttackType) -> &'static str {
    if atk.mana_cost > 0 {
        "green"
    } else if atk.vigor_cost > 0 {
        "orange"
    } else if atk.berseck_cost > 0 {
        "red"
    } else {
        "white"
    }
}

fn get_cost(atk: &AttackType) -> String {
    if atk.mana_cost > 0 {
        atk.mana_cost.to_string()
    } else if atk.vigor_cost > 0 {
        atk.vigor_cost.to_string()
    } else if atk.berseck_cost > 0 {
        atk.berseck_cost.to_string()
    } else {
        String::from("")
    }
}

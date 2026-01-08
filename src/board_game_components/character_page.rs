use colorgrad::Gradient;
use dioxus::prelude::*;
use indexmap::IndexMap;
use lib_rpg::{
    attack_type::AttackType,
    character::{Character, CharacterType},
    common::stats_const::*,
};

use crate::{
    application::log_debug,
    common::{APP, ENERGY_GRAD},
};
use crate::{
    common::PATH_IMG,
    components::button::{Button, ButtonVariant},
};

#[component]
pub fn CharacterPanel(
    c: Character,
    current_player_name: String,
    selected_atk_name: Signal<String>,
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
            Button {
                variant: ButtonVariant::AtkAutoMenu,
                onclick: move |_| async move {},
                "ATK On Going"
            }
        } else if kind2.clone() == CharacterType::Hero && current_player_name == name2.clone() {
            Button {
                variant: ButtonVariant::AtkMenu,
                onclick: move |_| async move {
                    atk_menu_display.set(!atk_menu_display());
                },
                "ATK"
            }
        }
        // name button
        Button {
            variant: ButtonVariant::CharacterName,
            onclick: move |_| async move {},
            "{name2.clone()}"
        }
        // target button
        if !selected_atk_name().is_empty() {
            CharacterTargetButton {
                launcher_name: current_player_name,
                c: c.clone(),
                selected_atk_name,
                write_game_manager,
            }
        }
    }
}

#[component]
pub fn CharacterTargetButton(
    launcher_name: String,
    c: Character,
    selected_atk_name: Signal<String>,
    write_game_manager: Signal<bool>,
) -> Element {
    let mut kind_str = "hero";
    if c.kind == CharacterType::Boss {
        kind_str = "boss";
    }
    rsx! {
        if c.is_current_target {
            Button {
                variant: ButtonVariant::Primary,
                class: format!("{}-target-button-active", kind_str),
                onclick: move |_| async move {},
                ""
            }
        } else if c.is_potential_target {
            Button {
                variant: ButtonVariant::Primary,
                class: format!("{}-target-button", kind_str),
                onclick: move |_| {
                    let async_target_name = c.name.clone();
                    let async_launcher_name = launcher_name.clone();
                    async move {
                        APP.write()
                            .game_manager
                            .pm
                            .set_one_target(
                                &async_launcher_name,
                                &selected_atk_name(),
                                &async_target_name,
                            );
                        log_debug(
                                format!(
                                    "l:{} t:{}, a:{}",
                                    async_launcher_name.clone(),
                                    async_target_name.clone(),
                                    selected_atk_name.read().clone(),
                                ),
                            )
                            .await
                            .unwrap();
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
    launcher: Character,
    write_game_manager: Signal<bool>,
    selected_atk_name: Signal<String>,
) -> Element {
    let can_be_launched = launcher.can_be_launched(&attack_type);
    let attack_name = attack_type.name.clone();
    let launcher_name = launcher.name;
    rsx! {
        Button {
            variant: if can_be_launched { ButtonVariant::AtkName } else { ButtonVariant::AtkNameBlocked },
            onclick: move |_| {
                let async_atk_name = attack_name.clone();
                let async_launcher_name = launcher_name.clone();
                async move {
                    selected_atk_name.set(async_atk_name.clone());
                    APP.write()
                        .game_manager
                        .pm
                        .set_targeted_characters(&async_launcher_name, &async_atk_name);
                    log_debug("set_targeted_characters".to_owned()).await.unwrap();
                    *display_atklist_sig.write() = false;
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
    write_game_manager: Signal<bool>,
    selected_atk_name: Signal<String>,
) -> Element {
    if let Some(c) = APP.read().game_manager.pm.get_active_character(&name) {
        rsx! {
            div { class: "attack-list",
                for (_key , value) in c.attacks_list.iter() {
                    if c.level >= value.level {
                        div { class: "attack-list-line",
                            Button {
                                variant: get_variant_atk_type(value),
                                onclick: move |_| {},
                                {get_cost(value)}
                            }
                            NewAtkButton {
                                attack_type: value.clone(),
                                display_atklist_sig,
                                launcher: c.clone(),
                                write_game_manager,
                                selected_atk_name,
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

fn get_variant_atk_type(atk: &AttackType) -> ButtonVariant {
    if atk.mana_cost > 0 {
        ButtonVariant::AtkManaType
    } else if atk.vigor_cost > 0 {
        ButtonVariant::AtkVigorType
    } else if atk.berseck_cost > 0 {
        ButtonVariant::AtkBerserkType
    } else {
        ButtonVariant::AtkDefaultType
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

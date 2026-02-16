use colorgrad::Gradient;
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_primitives::ContentSide;
use indexmap::IndexMap;
use lib_rpg::{
    attack_type::AttackType,
    character::{Character, CharacterType},
    character_mod::fight_information::{CharacterFightInfo, HotsBufs},
    common::stats_const::*,
};

use crate::{
    common::PATH_IMG,
    components::button::{Button, ButtonVariant},
};
use crate::{
    common::{ENERGY_GRAD, SERVER_NAME},
    components::tooltip::{Tooltip, TooltipContent, TooltipTrigger},
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        game_state::ServerData,
    },
};
use dioxus::logger::tracing;

#[component]
pub fn CharacterPanel(
    c: Character,
    current_player_name: String,
    selected_atk_name: Signal<String>,
    atk_menu_display: Signal<bool>,
    is_auto_atk: ReadSignal<bool>,
) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let local_session_player_name = use_context::<Signal<String>>();
    // get first player of the list
    let current_character = match server_data()
        .players_info
        .get(&local_session_player_name())
        .and_then(|info| info.character_names.first().cloned())
    {
        Some(player_name) => player_name,
        None => {
            tracing::error!(
                "No player found for session player name: {}",
                local_session_player_name()
            );
            String::new()
        }
    };
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

    rsx! {
        CharacterTooltip { hots_bufs: CharacterFightInfo::get_hot_and_buf_nbs_txts(&c.all_effects) }
        div { class: "character", background_color: bg,
            div {
                img {
                    src: format!("{}/{}.png", PATH_IMG, c.photo_name),
                    class: "image-small",
                }
            }
            div { class: "character-energy-effects-box",
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
        // atk button
        if is_auto_atk() {
            Button {
                variant: ButtonVariant::AtkAutoMenu,
                onclick: move |_| async move {},
                "ATK On Going"
            }
        } else if c.kind == CharacterType::Hero && current_player_name == c.name {
            Button {
                variant: ButtonVariant::AtkMenu,
                disabled: current_character != c.name,
                onclick: move |_| async move {
                    atk_menu_display.set(!atk_menu_display());
                },
                if current_character == c.name {
                    "ATK"
                } else {
                    "playing..."
                }
            }
        }
        div { class: "character-name-grid-container",
            // name buttons
            Button {
                variant: ButtonVariant::CharacterName,
                onclick: move |_| async move {},
                "{c.name} | Lvl: {c.level}"
            }
        }
        // target button
        if !selected_atk_name().is_empty() {
            CharacterTargetButton {
                launcher_name: current_player_name,
                c: c.clone(),
                selected_atk_name,
            }
        }
    }
}

#[component]
pub fn CharacterTargetButton(
    launcher_name: String,
    c: Character,
    selected_atk_name: Signal<String>,
) -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();

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
                        tracing::info!(
                            "l:{} t:{}, a:{}", async_launcher_name.clone(), async_target_name
                            .clone(), selected_atk_name.read().clone()
                        );
                        let _ = socket
                            .send(
                                ClientEvent::RequestSetOneTarget(
                                    SERVER_NAME(),
                                    async_launcher_name.clone(),
                                    selected_atk_name.read().clone(),
                                    async_target_name.clone(),
                                ),
                            )
                            .await;
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
    selected_atk_name: Signal<String>,
) -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    // local signals
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

                    *display_atklist_sig.write() = false;
                    // update target
                    let _ = socket
                        .send(
                            ClientEvent::RequestTargetedCharacter(
                                SERVER_NAME(),
                                async_launcher_name.clone(),
                                async_atk_name.clone(),
                            ),
                        )
                        .await;
                    tracing::info!(
                        "set_targeted_characters {} for atk {}", async_launcher_name.clone(),
                        async_atk_name.clone()
                    );
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
    selected_atk_name: Signal<String>,
) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    if let Some(c) = server_data()
        .app
        .game_manager
        .pm
        .get_active_character(&name)
    {
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

#[component]
fn CharacterTooltip(hots_bufs: HotsBufs) -> Element {
    rsx! {
        div { class: "character-effects",
            Tooltip {
                TooltipTrigger {
                    button {
                        height: "20px",
                        width: "20px",
                        background_color: "green",
                        "{hots_bufs.hot_nb}"
                    }
                    button {
                        height: "20px",
                        width: "20px",
                        background_color: "red",
                        "{hots_bufs.dot_nb}"
                    }
                    button {
                        height: "20px",
                        width: "20px",
                        background_color: "blue",
                        "{hots_bufs.buf_nb}"
                    }
                    button {
                        height: "20px",
                        width: "20px",
                        background_color: "orange",
                        "{hots_bufs.debuf_nb}"
                    }
                }
                TooltipContent { side: ContentSide::Right, style: "width: 300px;",
                    for txt in hots_bufs.hot_txt {
                        p { style: "margin: 0;", "hots: \n{txt}" }
                    }
                    for txt in hots_bufs.dot_txt {
                        p { style: "margin: 0;", "dots: \n{txt}" }
                    }
                    for txt in hots_bufs.buf_txt {
                        p { style: "margin: 0;", "bufs: \n{txt}" }
                    }
                    for txt in hots_bufs.debuf_txt {
                        p { style: "margin: 0;", "debufs: \n{txt}" }
                    }
                }
            }
        }

    }
}

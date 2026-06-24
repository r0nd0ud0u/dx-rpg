use colorgrad::Gradient;
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_primitives::ContentSide;
use indexmap::IndexMap;
use lib_rpg::{
    character_mod::{
        attack_type::AttackType,
        character::{Character, CharacterKind},
        energy::EnergyKind,
        inventory::ConsumableKind,
        rounds_information::{CharacterRoundsInfo, HotsBufs},
    },
    common::constants::stats_const::*,
    server::{game_manager::ResultLaunchAttack, server_manager::ServerData},
};

use crate::{
    common::photo_src,
    components::button::{Button, ButtonVariant},
};
use crate::{
    common::{
        CtxShowAtkTooltips, CtxShowBossEnergy, CtxShowBossHp, CtxShowHeroAggro,
        CtxToggleAtkAnimation, ENERGY_GRAD, SERVER_NAME,
    },
    components::tooltip::{Tooltip, TooltipContent, TooltipTrigger},
    websocket_handler::event::{ClientEvent, ServerEvent},
};
use dioxus::logger::tracing;

/// Process the css class for the attack animation based on the last attack result and the character id_name
fn process_css_class_on_atk(last_atk: &ResultLaunchAttack, id_name: &str) -> &'static str {
    // eval class css for animation
    let is_blinking = last_atk.new_game_atk_effects.iter().any(|effect| {
        effect.effect_outcome.target_id_name == id_name && effect.effect_outcome.full_amount_tx < 0
    });
    let is_dodging = last_atk
        .all_dodging
        .iter()
        .any(|dodge_info| dodge_info.name == id_name && dodge_info.is_dodging);
    let is_blocking = last_atk
        .all_dodging
        .iter()
        .any(|dodge_info| dodge_info.name == id_name && dodge_info.is_blocking);
    // Blocking takes priority so the jello animation always shows (even without damage)
    if is_blocking {
        return "jello-horizontal";
    }
    if is_dodging {
        return "wobble-hor-bottom";
    }
    if is_blinking {
        return "blink-1";
    }
    ""
}

#[component]
pub fn CharacterPanel(
    c: Character,
    current_player_id_name: String,
    selected_atk_name: Signal<String>,
    selected_consumable: Signal<String>,
    selected_consumable_target: Signal<String>,
    atk_menu_display: Signal<bool>,
    potion_menu_display: Signal<bool>,
    is_auto_atk: ReadSignal<bool>,
) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let local_session_player_name = use_context::<Signal<String>>();
    let toggle_atk_animation = use_context::<CtxToggleAtkAnimation>().0;
    let show_boss_energy = use_context::<CtxShowBossEnergy>().0;
    let show_hero_aggro = use_context::<CtxShowHeroAggro>().0;
    let show_boss_hp = use_context::<CtxShowBossHp>().0;
    // get first player of the list
    let current_character = {
        let sd = server_data();
        let is_single = sd.core_game_data.is_single_player;
        if is_single {
            // In single-player, the active player IS the current character
            sd.core_game_data
                .game_manager
                .pm
                .current_player
                .id_name
                .clone()
        } else {
            match sd
                .players_data
                .get_first_character_name(&local_session_player_name())
            {
                Some(player_name) => player_name,
                None => {
                    tracing::error!(
                        "No player found for session player name: {}",
                        local_session_player_name()
                    );
                    String::new()
                }
            }
        }
    };
    // if boss is dead, panel is hidden
    if c.stats.is_dead().is_some_and(|value| value) && c.kind == CharacterKind::Boss {
        return rsx! {};
    }
    let bg = if c.kind == CharacterKind::Hero {
        "var(--secondary-color-2)"
    } else {
        "var(--secondary-error-color)"
    };
    // Highlight the panel whose turn it is to play
    let is_active_player = c.id_name == current_player_id_name;
    let panel_border = if is_active_player {
        "2px solid var(--rpg-gold)"
    } else {
        "none"
    };
    let panel_box_shadow = if is_active_player {
        "0 0 12px 2px rgba(201,162,39,0.55)"
    } else {
        "none"
    };
    let energy_list = IndexMap::from([
        (MANA.to_owned(), ("MP".to_owned(), EnergyKind::Mana)),
        (VIGOR.to_owned(), ("VP".to_owned(), EnergyKind::Vigor)),
        (BERSERK.to_owned(), ("BP".to_owned(), EnergyKind::Berserk)),
    ]);

    // eval class css for animation
    let mut class_css = process_css_class_on_atk(
        &server_data()
            .core_game_data
            .game_manager
            .game_state
            .last_result_atk,
        &c.id_name,
    );
    if toggle_atk_animation() {
        class_css = "";
    }

    let extra_rounds = {
        let sd = server_data();
        sd.core_game_data
            .game_manager
            .game_state
            .order_to_play
            .iter()
            .filter(|id| **id == c.id_name)
            .count()
            .saturating_sub(1)
    };

    rsx! {
        div { class: class_css, position: "relative",
            CharacterTooltip {
                hots_bufs: CharacterRoundsInfo::get_hot_and_buf_nbs_txts(&c.character_rounds_info.all_effects),
                prefer_left: c.kind == CharacterKind::Boss,
            }
            div {
                class: "character",
                background_color: bg,
                border: panel_border,
                box_shadow: panel_box_shadow,
                // Header: name + level + attack button
                div { class: "char-header",
                    span { class: "char-name-text", "{c.db_full_name}" }
                    span { class: "char-level", "Lvl {c.level}" }
                    if extra_rounds > 0 {
                        span {
                            class: "char-extra-rounds",
                            title: "Extra round from speed advantage",
                            "⚡×{extra_rounds}"
                        }
                    }
                    if c.kind == CharacterKind::Hero && show_hero_aggro() {
                        if let Some(aggro_stat) = c.stats.all_stats.get(AGGRO) {
                            span { class: "char-aggro", title: "Aggro", "🎯 {aggro_stat.current}" }
                        }
                    }
                    if is_auto_atk() {
                        Button {
                            variant: ButtonVariant::AtkAutoMenu,
                            onclick: move |_| async move {},
                            "⏳🤖"
                        }
                    } else if c.kind == CharacterKind::Hero && current_player_id_name == c.id_name {
                        Button {
                            variant: ButtonVariant::AtkMenu,
                            disabled: current_character != c.id_name,
                            onclick: move |_| async move {
                                atk_menu_display.set(!atk_menu_display());
                                potion_menu_display.set(false);
                            },
                            if current_character == c.id_name {
                                "⚔️"
                            } else {
                                "⏳"
                            }
                        }
                        if current_character == c.id_name
                            && (!c.inventory.consumables.is_empty()
                                || !server_data()
                                    .core_game_data
                                    .game_manager
                                    .pm
                                    .party_consumables
                                    .is_empty())
                        {
                            Button {
                                variant: ButtonVariant::AtkMenu,
                                onclick: move |_| async move {
                                    potion_menu_display.set(!potion_menu_display());
                                    atk_menu_display.set(false);
                                },
                                "💊"
                            }
                        }
                    }
                }
                // Body: image + bars
                div { class: "char-body",
                    img { src: photo_src(&c.photo_name), class: "image-small" }
                    div { class: "character-energy-effects-box",
                        if c.kind == CharacterKind::Hero || (show_boss_hp() && c.kind == CharacterKind::Boss) {
                            BarComponent {
                                max: c.stats.all_stats[HP].max,
                                current: c.stats.all_stats[HP].current,
                                name: HP.to_owned(),
                            }
                        }
                        if c.kind == CharacterKind::Hero || show_boss_energy() {
                            for (stat, energy) in energy_list.iter() {
                                if c.stats.all_stats[stat].max > 0 && c.has_energy_kind(&energy.1) {
                                    BarComponent {
                                        max: c.stats.all_stats[stat].max,
                                        current: c.stats.all_stats[stat].current,
                                        name: energy.0.clone(),
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Target button (absolute positioned, stays outside card)
            if !selected_atk_name().is_empty() {
                CharacterTargetButton {
                    launcher_id_name: current_player_id_name,
                    c: c.clone(),
                    selected_atk_name,
                    selected_consumable,
                    selected_consumable_target,
                }
            } else if !selected_consumable().is_empty() {
                CharacterTargetButton {
                    launcher_id_name: current_player_id_name,
                    c: c.clone(),
                    selected_atk_name,
                    selected_consumable,
                    selected_consumable_target,
                }
            }
        }
    }
}

#[component]
pub fn CharacterTargetButton(
    launcher_id_name: String,
    c: Character,
    selected_atk_name: Signal<String>,
    selected_consumable: Signal<String>,
    selected_consumable_target: Signal<String>,
) -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_session_player_name = use_context::<Signal<String>>();

    let mut kind_str = "hero";
    if c.kind == CharacterKind::Boss {
        kind_str = "boss";
    }

    // In consumable mode: clicking only selects the target (fires on "Use" button).
    // In attack mode: clicking sets the attack target via RequestSetOneTarget.
    let is_consumable_mode = !selected_consumable().is_empty();

    // Active = this character is the current target.
    // For consumables, the local signal overrides the server flag.
    let is_active = if is_consumable_mode {
        if !selected_consumable_target().is_empty() {
            selected_consumable_target() == c.id_name
        } else {
            c.character_rounds_info.is_current_target
        }
    } else {
        c.character_rounds_info.is_current_target
    };

    let onclick = {
        let target_name = c.id_name.clone();
        let launcher_name = launcher_id_name.clone();
        move |_| {
            let target_name = target_name.clone();
            let launcher_name = launcher_name.clone();
            let async_player = local_session_player_name();
            async move {
                if is_consumable_mode {
                    // Just update the local selection — no server call until "Use" is clicked.
                    selected_consumable_target.set(target_name);
                } else {
                    tracing::info!(
                        "l:{} t:{}, a:{}",
                        launcher_name,
                        target_name,
                        selected_atk_name.read().clone()
                    );
                    let _ = socket
                        .send(ClientEvent::RequestSetOneTarget(
                            SERVER_NAME(),
                            launcher_name,
                            selected_atk_name.read().clone(),
                            target_name,
                        ))
                        .await;
                    let _ = async_player; // suppress unused warning
                }
            }
        }
    };

    rsx! {
        if is_active && c.character_rounds_info.is_potential_target {
            Button {
                variant: ButtonVariant::Primary,
                class: format!("{}-target-button-active", kind_str),
                onclick,
                ""
            }
        } else if c.character_rounds_info.is_potential_target {
            Button {
                variant: ButtonVariant::Primary,
                class: format!("{}-target-button", kind_str),
                onclick,
                ""
            }
        }
    }
}

#[component]
pub fn BarComponent(max: u64, current: u64, name: String) -> Element {
    let width_display = (current * 100).checked_div(max).unwrap_or(0);
    rsx! {
        div { class: "bar-row",
            div { class: "bar-header",
                span { class: "bar-name", "{name}" }
                span { class: "bar-value", "{current}/{max}" }
            }
            div { class: "container-bar",
                div {
                    class: "life-bar",
                    width: "{width_display}%",
                    background_color: get_color(width_display as i32),
                }
            }
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
    let show_tooltips = use_context::<CtxShowAtkTooltips>().0;
    // local signals
    let can_be_launched = launcher
        .character_rounds_info
        .launchable_atks
        .iter()
        .any(|atk| atk.name == attack_type.name);
    let attack_name = attack_type.name.clone();
    let launcher_id_name = launcher.id_name.clone();
    let description = attack_type.description.clone();
    let effects_description = attack_type.effects_description.clone();
    let has_description = !description.is_empty();
    let has_effects = !effects_description.is_empty();
    let has_tooltip = has_description || has_effects;
    rsx! {
        Tooltip { disabled: !has_tooltip || !show_tooltips(),
            TooltipTrigger {
                Button {
                    variant: if can_be_launched { ButtonVariant::AtkName } else { ButtonVariant::AtkNameBlocked },
                    onclick: move |_| {
                        let async_atk_name = attack_name.clone();
                        let async_launcher_name = launcher_id_name.clone();
                        async move {
                            selected_atk_name.set(async_atk_name.clone());
                            *display_atklist_sig.write() = false;
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
            TooltipContent {
                p { style: "margin:0 0 4px 0; font-weight:600; color:var(--rpg-gold,#c9a227);",
                    "{attack_type.name}"
                }
                if has_description {
                    p { style: "margin:0; color:var(--primary-color); font-style:italic;",
                        "{description}"
                    }
                }
                if has_effects {
                    div { style: "margin-top:6px; padding-top:6px; border-top:1px solid var(--rpg-border,#3a3f55);",
                        span { style: "display:block; margin-bottom:2px; font-size:0.72rem; font-weight:600; letter-spacing:0.04em; text-transform:uppercase; color:var(--rpg-text-muted,#8a8fa8);",
                            "Effects"
                        }
                        p { style: "margin:0; line-height:1.4;", "{effects_description}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AttackList(
    id_name: String,
    display_atklist_sig: Signal<bool>,
    selected_atk_name: Signal<String>,
) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();

    if let Some(c) = server_data()
        .core_game_data
        .game_manager
        .pm
        .get_active_character(&id_name)
    {
        rsx! {
            div { class: "attack-list",
                for (_key, value) in c.attacks_list.iter() {
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

#[component]
pub fn PotionList(
    id_name: String,
    display_potionlist_sig: Signal<bool>,
    selected_consumable: Signal<String>,
) -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_session_player_name = use_context::<Signal<String>>();

    let snap = server_data();

    // Personal potions
    let potions: Vec<String> = snap
        .core_game_data
        .game_manager
        .pm
        .get_active_character(&id_name)
        .map(|c| {
            c.inventory
                .consumables
                .iter()
                .filter(|c| c.consumable_kind == ConsumableKind::Potion)
                .map(|c| c.name.clone())
                .collect()
        })
        .unwrap_or_default();

    // Party bag potions
    let party_potions: Vec<String> = snap
        .core_game_data
        .game_manager
        .pm
        .party_consumables
        .iter()
        .map(|c| c.name.clone())
        .collect();

    fn group_by_name(names: &[String]) -> (Vec<String>, std::collections::HashMap<String, usize>) {
        let mut seen_order: Vec<String> = Vec::new();
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for name in names {
            let entry = counts.entry(name.clone()).or_insert(0);
            if *entry == 0 {
                seen_order.push(name.clone());
            }
            *entry += 1;
        }
        (seen_order, counts)
    }

    let (personal_order, personal_counts) = group_by_name(&potions);
    let (party_order, party_counts) = group_by_name(&party_potions);

    let is_empty = personal_order.is_empty() && party_order.is_empty();
    if is_empty {
        return rsx! {
            div { class: "attack-list",
                span { style: "color: var(--rpg-text-muted); font-size: 0.85rem;",
                    "No potions available"
                }
            }
        };
    }

    rsx! {
        div { class: "attack-list",
            // ── Personal potions ──────────────────────────────────────────────
            if !personal_order.is_empty() {
                span { class: "potion-bag-header", "💊 Personal potion" }
                for potion_name in personal_order {
                    {
                        let count = personal_counts[&potion_name];
                        let label = if count > 1 {
                            format!("💊 {} ×{}", potion_name, count)
                        } else {
                            format!("💊 {}", potion_name)
                        };
                        rsx! {
                            Button {
                                variant: ButtonVariant::AtkName,
                                onclick: {
                                    let pname = potion_name.clone();
                                    let player = local_session_player_name();
                                    move |_| {
                                        let async_potion = pname.clone();
                                        let async_player = player.clone();
                                        async move {
                                            let _ = socket
                                                .send(
                                                    ClientEvent::RequestTargetForConsumable(
                                                        SERVER_NAME(),
                                                        async_player,
                                                        async_potion.clone(),
                                                        false,
                                                    ),
                                                )
                                                .await;
                                            selected_consumable.set(format!("personal:{}", async_potion));
                                            display_potionlist_sig.set(false);
                                        }
                                    }
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
            // ── Common consumables (shared party bag) ────────────────────────────────────────
            if !party_order.is_empty() {
                span { class: "potion-bag-header", "🎁 Common consumable" }
                for potion_name in party_order {
                    {
                        let count = party_counts[&potion_name];
                        let label = if count > 1 {
                            format!("✨ {} ×{}", potion_name, count)
                        } else {
                            format!("✨ {}", potion_name)
                        };
                        rsx! {
                            Button {
                                variant: ButtonVariant::AtkName,
                                onclick: {
                                    let pname = potion_name.clone();
                                    let player = local_session_player_name();
                                    move |_| {
                                        let async_potion = pname.clone();
                                        let async_player = player.clone();
                                        async move {
                                            let _ = socket
                                                .send(
                                                    ClientEvent::RequestTargetForConsumable(
                                                        SERVER_NAME(),
                                                        async_player,
                                                        async_potion.clone(),
                                                        true,
                                                    ),
                                                )
                                                .await;
                                            selected_consumable.set(format!("party:{}", async_potion));
                                            display_potionlist_sig.set(false);
                                        }
                                    }
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
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
fn CharacterTooltip(hots_bufs: HotsBufs, prefer_left: bool) -> Element {
    let has_effects = hots_bufs.hot_nb > 0
        || hots_bufs.dot_nb > 0
        || hots_bufs.buf_nb > 0
        || hots_bufs.debuf_nb > 0;
    if !has_effects {
        return rsx! {};
    }
    let side = if prefer_left {
        ContentSide::Left
    } else {
        ContentSide::Right
    };
    rsx! {
        div { class: "character-effects",
            Tooltip {
                TooltipTrigger {
                    div { style: "display:flex; flex-direction:row; gap:3px;",
                        if hots_bufs.hot_nb > 0 {
                            span { class: "effect-badge effect-hot", "🌿 {hots_bufs.hot_nb}" }
                        }
                        if hots_bufs.dot_nb > 0 {
                            span { class: "effect-badge effect-dot", "🔥 {hots_bufs.dot_nb}" }
                        }
                        if hots_bufs.buf_nb > 0 {
                            span { class: "effect-badge effect-buf", "⬆ {hots_bufs.buf_nb}" }
                        }
                        if hots_bufs.debuf_nb > 0 {
                            span { class: "effect-badge effect-debuf", "⬇ {hots_bufs.debuf_nb}" }
                        }
                    }
                }
                TooltipContent { side,
                    for txt in hots_bufs.hot_txt {
                        p { style: "margin: 0;", "🌿 {txt}" }
                    }
                    for txt in hots_bufs.dot_txt {
                        p { style: "margin: 0;", "🔥 {txt}" }
                    }
                    for txt in hots_bufs.buf_txt {
                        if txt.contains(": cooldown (") {
                            p { style: "margin: 0;", "⏳ {txt}" }
                        } else {
                            p { style: "margin: 0;", "⬆ {txt}" }
                        }
                    }
                    for txt in hots_bufs.debuf_txt {
                        p { style: "margin: 0;", "⬇ {txt}" }
                    }
                }
            }
        }
    }
}

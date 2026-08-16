use colorgrad::Gradient;
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_i18n::t;
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
    auth_manager::server_fn::{get_user_setting, save_user_setting},
    common::photo_src,
    components::button::{Button, ButtonVariant},
};
use crate::{
    common::{
        CtxAppLang, CtxAtkPanelOrders, CtxShowAtkTooltips, CtxShowBossEnergy, CtxShowBossHp,
        CtxShowHeroAggro, CtxToggleAtkAnimation, ENERGY_GRAD, SERVER_NAME, lang_from_app_lang,
    },
    components::{
        drag_and_drop_list::{DragAndDropList, use_drag_and_drop_list_order},
        sheet::{Sheet, SheetContent, SheetDescription, SheetHeader, SheetSide, SheetTitle},
        tooltip::{Tooltip, TooltipContent, TooltipTrigger},
    },
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
    let app_lang = use_context::<CtxAppLang>().0;
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
                hots_bufs: CharacterRoundsInfo::get_hot_and_buf_nbs_txts(
                    &c.character_rounds_info.all_effects,
                    lang_from_app_lang(&app_lang()),
                ),
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
                    span { class: "char-level", {t!("character-page-lvl", level : c.level as i64)} }
                    if extra_rounds > 0 {
                        span {
                            class: "char-extra-rounds",
                            title: t!("character-page-extra-round-title"),
                            "⚡×{extra_rounds}"
                        }
                    }
                    if c.kind == CharacterKind::Hero && show_hero_aggro() {
                        if let Some(aggro_stat) = c.stats.all_stats.get(AGGRO) {
                            span {
                                class: "char-aggro",
                                title: t!("character-page-aggro-title"),
                                "🎯 {aggro_stat.current}"
                            }
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
    let app_lang = use_context::<CtxAppLang>().0;
    // local signals
    let can_be_launched = launcher
        .character_rounds_info
        .launchable_atks
        .iter()
        .any(|atk| atk.name == attack_type.name);
    let attack_name = attack_type.name.clone();
    let launcher_id_name = launcher.id_name.clone();
    let lang = lang_from_app_lang(&app_lang());
    let display_name = attack_type.name_for(lang).to_string();
    let description = attack_type.description_for(lang).to_string();
    let effects_description = attack_type.effects_description_for(lang).to_string();
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
                    "{display_name}"
                }
            }
            TooltipContent {
                p { style: "margin:0 0 4px 0; font-weight:600; color:var(--rpg-gold,#c9a227);",
                    "{display_name}"
                }
                if has_description {
                    p { style: "margin:0; color:var(--primary-color); font-style:italic;",
                        "{description}"
                    }
                }
                if has_effects {
                    div { style: "margin-top:6px; padding-top:6px; border-top:1px solid var(--rpg-border,#3a3f55);",
                        span { style: "display:block; margin-bottom:2px; font-size:0.72rem; font-weight:600; letter-spacing:0.04em; text-transform:uppercase; color:var(--rpg-text-muted,#8a8fa8);",
                            {t!("character-page-effects-label")}
                        }
                        p { style: "margin:0; line-height:1.4;", "{effects_description}" }
                    }
                }
            }
        }
    }
}

/// Key under which a player's custom attack panel order is stored in
/// `user_settings`, scoped to this game/server and this character — so the
/// same player can have a different layout per hero and per server.
fn atk_panel_order_key(server_name: &str, character_id: &str) -> String {
    format!("atk_panel_order:{server_name}:{character_id}")
}

fn serialize_order(order: &[String]) -> String {
    serde_json::to_string(order).unwrap_or_default()
}

fn parse_order(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

/// Reorders `attacks` to match `order` (matched by `AttackType::name`).
/// Attacks not listed in `order` — e.g. newly unlocked since the order was
/// saved — are appended afterward in their original order, so nothing is
/// ever silently dropped from the panel.
fn apply_custom_order<'a>(attacks: &[&'a AttackType], order: &[String]) -> Vec<&'a AttackType> {
    if order.is_empty() {
        return attacks.to_vec();
    }
    let mut ordered: Vec<&'a AttackType> = Vec::with_capacity(attacks.len());
    for name in order {
        if let Some(atk) = attacks.iter().find(|a| &a.name == name) {
            ordered.push(*atk);
        }
    }
    for atk in attacks {
        if !ordered.iter().any(|a| a.name == atk.name) {
            ordered.push(atk);
        }
    }
    ordered
}

/// Ephemeral, display-only sort applied on top of the saved custom order —
/// never persisted (see `AttackList`/`AttackPanelConfig`).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum AtkSortMode {
    #[default]
    Natural,
    Level,
    Cost,
}

fn atk_cost_value(atk: &AttackType) -> u64 {
    if atk.mana_cost > 0 {
        atk.mana_cost
    } else if atk.vigor_cost > 0 {
        atk.vigor_cost
    } else {
        atk.berseck_cost
    }
}

fn sort_attacks(mut attacks: Vec<&AttackType>, mode: AtkSortMode) -> Vec<&AttackType> {
    match mode {
        AtkSortMode::Natural => {}
        AtkSortMode::Level => attacks.sort_by_key(|a| a.level),
        AtkSortMode::Cost => attacks.sort_by_key(|a| atk_cost_value(a)),
    }
    attacks
}

#[component]
pub fn AttackList(
    id_name: String,
    display_atklist_sig: Signal<bool>,
    selected_atk_name: Signal<String>,
) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let mut atk_panel_orders = use_context::<CtxAtkPanelOrders>().0;
    let mut sort_mode = use_signal(AtkSortMode::default);
    let mut config_open = use_signal(|| false);

    // Load this character's saved custom order once, on demand.
    use_effect({
        let id_name = id_name.clone();
        move || {
            let id_name = id_name.clone();
            if atk_panel_orders.read().contains_key(&id_name) {
                return;
            }
            let server_name = SERVER_NAME();
            spawn(async move {
                let key = atk_panel_order_key(&server_name, &id_name);
                if let Ok(raw) = get_user_setting(key, String::new()).await {
                    let order = parse_order(&raw);
                    atk_panel_orders.write().insert(id_name, order);
                }
            });
        }
    });

    if let Some(c) = server_data()
        .core_game_data
        .game_manager
        .pm
        .get_active_character(&id_name)
    {
        let filtered: Vec<&AttackType> = c
            .attacks_list
            .values()
            .filter(|value| c.level >= value.level)
            .collect();
        let custom_order = atk_panel_orders
            .read()
            .get(&id_name)
            .cloned()
            .unwrap_or_default();
        let ordered = apply_custom_order(&filtered, &custom_order);
        let displayed = sort_attacks(ordered, sort_mode());
        let config_attacks: Vec<AttackType> = filtered.iter().map(|a| (*a).clone()).collect();

        rsx! {
            div { class: "attack-list",
                div { class: "attack-list-toolbar",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| sort_mode.set(AtkSortMode::Level),
                        {t!("character-page-sort-by-level")}
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| sort_mode.set(AtkSortMode::Cost),
                        {t!("character-page-sort-by-cost")}
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| config_open.set(true),
                        {t!("character-page-configure-atk-panel")}
                    }
                }
                for value in displayed.iter() {
                    div { class: "attack-list-line",
                        Button {
                            variant: get_variant_atk_type(value),
                            onclick: move |_| {},
                            {get_cost(value)}
                        }
                        NewAtkButton {
                            attack_type: (*value).clone(),
                            display_atklist_sig,
                            launcher: c.clone(),
                            selected_atk_name,
                        }
                    }
                }
            }
            AttackPanelConfig {
                id_name: id_name.clone(),
                attacks: config_attacks,
                open: config_open,
            }
        }
    } else {
        rsx! {}
    }
}

/// Reads the live drag-and-drop order from within the `DragAndDropList`'s
/// context subtree and mirrors it out to `order_out`, a signal owned by
/// `AttackPanelConfig` (which sits outside that subtree and so can't read
/// the order itself — see `use_drag_and_drop_list_order`).
#[component]
fn AtkOrderReadout(mut order_out: Signal<Vec<String>>) -> Element {
    let live_order = use_drag_and_drop_list_order();
    use_effect(move || {
        order_out.set(live_order());
    });
    rsx! {}
}

#[component]
fn AttackPanelConfig(id_name: String, attacks: Vec<AttackType>, open: Signal<bool>) -> Element {
    let mut atk_panel_orders = use_context::<CtxAtkPanelOrders>().0;
    let mut draft: Signal<Vec<AttackType>> = use_signal(Vec::new);
    // Bumped to force the DragAndDropList to remount with a fresh initial
    // order after a "Sort by ..." shortcut — its internal order is only
    // seeded from `items`/`item_keys` once, at mount.
    let mut remount_key = use_signal(|| 0u32);
    let live_order: Signal<Vec<String>> = use_signal(Vec::new);
    let app_lang = use_context::<CtxAppLang>().0;
    let lang = lang_from_app_lang(&app_lang());

    let effective_order = {
        let id_name = id_name.clone();
        let attacks = attacks.clone();
        move || {
            let custom_order = atk_panel_orders
                .read()
                .get(&id_name)
                .cloned()
                .unwrap_or_default();
            let refs: Vec<&AttackType> = attacks.iter().collect();
            apply_custom_order(&refs, &custom_order)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    // Re-seed the draft from the current effective order every time the
    // sheet opens, so Cancel never leaves a stale draft for next time.
    use_effect({
        let effective_order = effective_order.clone();
        move || {
            if !open() {
                return;
            }
            draft.set(effective_order());
            // `.peek()`, not `()`: reading remount_key reactively here would make this
            // same effect depend on the write below, re-triggering itself forever.
            let next_remount_key = remount_key.peek().wrapping_add(1);
            remount_key.set(next_remount_key);
        }
    });

    // The effect above only runs *after* the render that flips `open` to
    // true commits, so on a freshly-mounted panel (`draft` still empty) the
    // very first paint would otherwise show an empty drag list for a frame
    // before the effect populates it and force-remounts the list — a visible
    // flash on top of the sheet's normal slide-in animation. Falling back to
    // computing it inline here means that first paint is already correct.
    if draft.peek().is_empty() && open() {
        draft.set(effective_order());
    }

    // Native HTML5 drag-and-drop (`draggable`/`ondragstart`/`ondragover`) has
    // no touch-input equivalent on Android's WebView (or any mobile
    // browser) — a finger drag never fires `dragstart`. The list's keyboard
    // fallback (Enter to grab, arrow keys to move) doesn't help there
    // either since a touch tap on a non-`<input>` element doesn't summon a
    // directional on-screen keyboard. So on top of the drag handle (mouse,
    // desktop-only in practice), every row also gets explicit Up/Down
    // buttons that directly mutate `draft` and force a remount — the same
    // mechanism the "Sort by ..." shortcuts already use — giving touch
    // users a working way to reorder.
    let items_len = draft().len();
    let items: Vec<Element> = draft()
        .iter()
        .enumerate()
        .map(|(index, atk)| {
            let label = atk.name_for(lang).to_string();
            rsx! {
                div { class: "atk-dnd-item-row",
                    span { class: "atk-dnd-item-label", "{label}" }
                    div { class: "atk-dnd-item-reorder",
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: index == 0,
                            onclick: move |evt: MouseEvent| {
                                evt.stop_propagation();
                                let mut current = draft();
                                if index > 0 {
                                    current.swap(index, index - 1);
                                    draft.set(current);
                                    remount_key.set(remount_key() + 1);
                                }
                            },
                            "▲"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: index + 1 >= items_len,
                            onclick: move |evt: MouseEvent| {
                                evt.stop_propagation();
                                let mut current = draft();
                                if index + 1 < current.len() {
                                    current.swap(index, index + 1);
                                    draft.set(current);
                                    remount_key.set(remount_key() + 1);
                                }
                            },
                            "▼"
                        }
                    }
                }
            }
        })
        .collect();
    let item_keys: Vec<String> = draft().iter().map(|atk| atk.name.clone()).collect();

    rsx! {
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            // Children below are ordered by CSS `order` rather than source
            // position: `key` (needed to force DragAndDropList to remount
            // with a fresh initial order after a "Sort by ..." shortcut) is
            // only allowed on the first root node of this block.
            SheetContent { side: SheetSide::Bottom,
                div {
                    style: "order: 2; display: flex; flex-direction: column;",
                    key: "{remount_key()}",
                    DragAndDropList {
                        items,
                        item_keys,
                        aria_label: t!("character-page-configure-atk-panel-title"),
                        AtkOrderReadout { order_out: live_order }
                    }
                }
                div { style: "order: 1;",
                    SheetHeader {
                        SheetTitle { {t!("character-page-configure-atk-panel-title")} }
                        SheetDescription { {t!("character-page-configure-atk-panel-desc")} }
                    }
                }
                div { class: "attack-list-toolbar", style: "order: 1;",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let current: Vec<AttackType> = draft();
                            let sorted = sort_attacks(current.iter().collect(), AtkSortMode::Level);
                            draft.set(sorted.into_iter().cloned().collect());
                            remount_key.set(remount_key() + 1);
                        },
                        {t!("character-page-sort-by-level")}
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let current: Vec<AttackType> = draft();
                            let sorted = sort_attacks(current.iter().collect(), AtkSortMode::Cost);
                            draft.set(sorted.into_iter().cloned().collect());
                            remount_key.set(remount_key() + 1);
                        },
                        {t!("character-page-sort-by-cost")}
                    }
                }
                div { class: "attack-list-toolbar", style: "order: 3;",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| open.set(false),
                        {t!("character-page-cancel")}
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: {
                            let id_name = id_name.clone();
                            move |_| {
                                let id_name = id_name.clone();
                                spawn(async move {
                                    let key = atk_panel_order_key(&SERVER_NAME(), &id_name);
                                    let _ = save_user_setting(key, String::new()).await;
                                    atk_panel_orders.write().remove(&id_name);
                                });
                                open.set(false);
                            }
                        },
                        {t!("character-page-reset-atk-panel")}
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: {
                            let id_name = id_name.clone();
                            move |_| {
                                let id_name = id_name.clone();
                                let order = live_order();
                                spawn(async move {
                                    let key = atk_panel_order_key(&SERVER_NAME(), &id_name);
                                    let value = serialize_order(&order);
                                    let _ = save_user_setting(key, value).await;
                                    atk_panel_orders.write().insert(id_name, order);
                                });
                                open.set(false);
                            }
                        },
                        {t!("character-page-save-atk-panel")}
                    }
                }
            }
        }
    }
}

fn get_color(value: i32) -> String {
    ENERGY_GRAD.at(value as f32 / 100.0).to_css_hex()
}

/// Groups a (possibly repeated) list of item names into display order + per-name counts,
/// e.g. `["Potion", "Potion", "Ether"]` -> `(["Potion", "Ether"], {"Potion": 2, "Ether": 1})`.
pub(crate) fn group_by_name(
    names: &[String],
) -> (Vec<String>, std::collections::HashMap<String, usize>) {
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

    let (personal_order, personal_counts) = group_by_name(&potions);
    let (party_order, party_counts) = group_by_name(&party_potions);

    let is_empty = personal_order.is_empty() && party_order.is_empty();
    if is_empty {
        return rsx! {
            div { class: "attack-list",
                span { style: "color: var(--rpg-text-muted); font-size: 0.85rem;",
                    {t!("character-page-no-potions")}
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

#[cfg(test)]
mod tests {
    use super::*;

    fn atk(name: &str, level: u64, mana: u64) -> AttackType {
        AttackType {
            name: name.to_owned(),
            level,
            mana_cost: mana,
            ..Default::default()
        }
    }

    #[test]
    fn atk_panel_order_key_scopes_by_server_and_character() {
        assert_eq!(
            atk_panel_order_key("server-1", "hero-1"),
            "atk_panel_order:server-1:hero-1"
        );
        assert_ne!(
            atk_panel_order_key("server-1", "hero-1"),
            atk_panel_order_key("server-2", "hero-1")
        );
        assert_ne!(
            atk_panel_order_key("server-1", "hero-1"),
            atk_panel_order_key("server-1", "hero-2")
        );
    }

    #[test]
    fn serialize_then_parse_order_round_trips() {
        let order = vec!["fireball".to_owned(), "heal".to_owned(), "slash".to_owned()];
        let raw = serialize_order(&order);
        assert_eq!(parse_order(&raw), order);
    }

    #[test]
    fn parse_order_on_garbage_or_empty_input_yields_empty() {
        assert_eq!(parse_order(""), Vec::<String>::new());
        assert_eq!(parse_order("not json"), Vec::<String>::new());
        assert_eq!(parse_order("{\"not\":\"an array\"}"), Vec::<String>::new());
    }

    #[test]
    fn apply_custom_order_with_empty_order_keeps_natural_order() {
        let fireball = atk("fireball", 1, 5);
        let heal = atk("heal", 2, 3);
        let attacks = vec![&fireball, &heal];
        let ordered = apply_custom_order(&attacks, &[]);
        assert_eq!(
            ordered.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["fireball", "heal"]
        );
    }

    #[test]
    fn apply_custom_order_reorders_by_saved_names() {
        let fireball = atk("fireball", 1, 5);
        let heal = atk("heal", 2, 3);
        let slash = atk("slash", 1, 0);
        let attacks = vec![&fireball, &heal, &slash];
        let order = vec!["slash".to_owned(), "fireball".to_owned(), "heal".to_owned()];
        let ordered = apply_custom_order(&attacks, &order);
        assert_eq!(
            ordered.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["slash", "fireball", "heal"]
        );
    }

    #[test]
    fn apply_custom_order_appends_attacks_missing_from_saved_order() {
        let fireball = atk("fireball", 1, 5);
        let heal = atk("heal", 2, 3);
        let new_atk = atk("new_atk", 1, 0);
        let attacks = vec![&fireball, &heal, &new_atk];
        // Saved before `new_atk` was unlocked/added.
        let order = vec!["heal".to_owned(), "fireball".to_owned()];
        let ordered = apply_custom_order(&attacks, &order);
        assert_eq!(
            ordered.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["heal", "fireball", "new_atk"]
        );
    }

    #[test]
    fn apply_custom_order_ignores_stale_names_no_longer_present() {
        let fireball = atk("fireball", 1, 5);
        let attacks = vec![&fireball];
        let order = vec!["removed_atk".to_owned(), "fireball".to_owned()];
        let ordered = apply_custom_order(&attacks, &order);
        assert_eq!(
            ordered.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["fireball"]
        );
    }

    #[test]
    fn sort_attacks_by_level_then_by_cost() {
        let fireball = atk("fireball", 3, 10);
        let heal = atk("heal", 1, 3);
        let slash = atk("slash", 2, 0);
        let attacks = vec![&fireball, &heal, &slash];

        let by_level = sort_attacks(attacks.clone(), AtkSortMode::Level);
        assert_eq!(
            by_level.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["heal", "slash", "fireball"]
        );

        let by_cost = sort_attacks(attacks, AtkSortMode::Cost);
        assert_eq!(
            by_cost.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["slash", "heal", "fireball"]
        );
    }
}

use std::collections::BTreeMap;

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_primitives::scroll_area::ScrollDirection;
use lib_rpg::{
    character_mod::{character::Character, stats::Attribute},
    common::{
        constants::stats_const::HP,
        log_data::{
            LogData,
            const_colors::{DARK_RED, LIGHT_BLUE, LIGHT_GREEN},
        },
    },
    server::{scenario::ScenarioState, server_manager::ServerData},
};

use crate::{
    auth_manager::server_fn::{get_user_setting, save_user_setting},
    components::{
        button::{Button, ButtonVariant},
        label::Label,
        scroll_area::ScrollArea,
        separator::Separator,
        sheet::{
            Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
            SheetTitle,
        },
        tabs::{TabContent, TabList, TabTrigger, Tabs},
    },
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::request_save_game,
    },
    widgets::{charts::TabStats, tab_equipment::TabEquipment},
};

#[component]
fn SaveButton(is_saved: Signal<bool>) -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    rsx! {
        Button {
            variant: ButtonVariant::Destructive,
            onclick: move |_| {
                async move {
                    request_save_game(socket, &local_login_name_session()).await;
                    is_saved.set(true);
                }
            },
            "Save"
        }
    }
}

#[derive(Clone, PartialEq)]
enum SheetKind {
    Menu,
    Inventory,
    Logs,
    Stats,
    Scenarios,
    Settings,
}

#[component]
pub fn GameSheets() -> Element {
    let mut open = use_signal(|| false);
    let mut sheet_kind: Signal<SheetKind> = use_signal(|| SheetKind::Menu);
    let mut is_saved: Signal<bool> = use_signal(|| false);

    let open_sheet = move |kind: SheetKind| {
        move |_| {
            sheet_kind.set(kind.clone());
            open.set(true);
        }
    };
    if !open() {
        is_saved.set(false);
    }

    rsx! {
        div { display: "flex", gap: "0.5rem",
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetKind::Menu),
                "Menu"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetKind::Inventory),
                "Inventory"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetKind::Logs),
                "Logs"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetKind::Stats),
                "Game stats"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetKind::Scenarios),
                "📜 Scenarios"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetKind::Settings),
                "⚙️ Settings"
            }
        }
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            match sheet_kind() {
                SheetKind::Inventory => {
                    InventorySheet(InventorySheetProps {
                        s: SheetSide::Right,
                    })
                }
                SheetKind::Stats => {
                    GameStatsSheet(GameStatsSheetProps {
                        s: SheetSide::Left,
                    })
                }
                SheetKind::Menu => {
                    MenuSheet(MenuSheetProps {
                        s: SheetSide::Top,
                        open_wnd: open,
                        is_saved,
                    })
                }
                SheetKind::Logs => {
                    LogsSheet(LogsSheetProps {
                        s: SheetSide::Bottom,
                    })
                }
                SheetKind::Scenarios => {
                    ScenariosSheet(ScenariosSheetProps {
                        s: SheetSide::Right,
                    })
                }
                SheetKind::Settings => {
                    SettingsSheet(SettingsSheetProps {
                        s: SheetSide::Left,
                    })
                }
            }
        }
    }
}

#[component]
fn InventorySheet(s: SheetSide) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    // snap
    let server_data_snap = server_data();

    // get character by name
    let Some(character_name) = server_data_snap
        .players_data
        .get_first_character_name(&local_login_name_session())
    else {
        return rsx! {};
    };
    let character = match server_data_snap
        .core_game_data
        .game_manager
        .pm
        .get_active_hero_character(&character_name)
    {
        Some(c) => c.clone(),
        None => Character::default(),
    };

    // BTreeMap — all stats sorted
    let ordered_stats: BTreeMap<String, Attribute> =
        character.stats.all_stats.clone().into_iter().collect();

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "📦 Inventory — {character.db_full_name}" }
                SheetDescription { "Level {character.level} · Stats & Equipment" }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "1rem",
                padding: "0 1rem",

                // Stats grid — 2 columns
                div {
                    display: "grid",
                    grid_template_columns: "1fr 1fr",
                    gap: "0.25rem 1.5rem",
                    for (k, v) in ordered_stats.iter() {
                        div {
                            display: "flex",
                            justify_content: "space-between",
                            align_items: "center",
                            padding: "3px 0",
                            border_bottom: "1px solid var(--rpg-border)",
                            Label {
                                html_for: "stat",
                                font_size: "0.78rem",
                                color: "var(--rpg-text-muted)",
                                "{k}"
                            }
                            Label {
                                html_for: "stat-val",
                                font_size: "0.78rem",
                                font_weight: "600",
                                "{v.current}/{v.max}"
                            }
                        }
                    }
                }

                Separator { horizontal: true, decorative: true }

                // Equipment tabs
                TabEquipment { c: character.clone() }
            }

            SheetFooter {
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Close" }
                    },
                }
            }
        }
    }
}

#[component]
fn GameStatsSheet(s: SheetSide) -> Element {
    let server_data = use_context::<Signal<ServerData>>();

    let snap = server_data();
    let gm = &snap.core_game_data.game_manager;
    let game_state = &gm.game_state;
    let current_player = gm.pm.current_player.id_name.clone();
    let current_round = game_state.current_round;
    let total_in_round = game_state.order_to_play.len();
    let current_turn = game_state.current_turn_nb;

    // Current scenario info
    let current_scenario = gm.all_scenarios.iter().find(|s| {
        matches!(
            gm.states_scenarios.get(&s.name),
            Some(ScenarioState::InProgress)
        )
    });

    // Kill count (dead bosses)
    let kills = gm
        .pm
        .all_bosses
        .iter()
        .filter(|b| b.stats.is_dead().unwrap_or(false))
        .count();
    let total_bosses_ever = gm.pm.all_bosses.len();

    // Scenario progress: completed scenarios
    let completed = gm
        .states_scenarios
        .values()
        .filter(|st| **st == ScenarioState::Completed)
        .count();
    let total_scenarios = gm.all_scenarios.len();

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "📊 Game Stats" }
                SheetDescription { "Evolution of the current game." }
            }

            ScrollArea {
                width: "100%",
                height: "calc(100vh - 10rem)",
                direction: ScrollDirection::Vertical,

                div {
                    display: "flex",
                    flex_direction: "column",
                    gap: "1rem",
                    padding: "0 1rem",

                    // ── Turn / Round / Scenario row ──────────────────────────
                    div {
                        display: "grid",
                        grid_template_columns: "1fr 1fr 1fr",
                        gap: "0.5rem",
                        div { class: "stats-kpi",
                            span { class: "stats-kpi-label", "TURN" }
                            span { class: "stats-kpi-value", "{current_turn}" }
                        }
                        div { class: "stats-kpi",
                            span { class: "stats-kpi-label", "ROUND" }
                            span { class: "stats-kpi-value", "{current_round}/{total_in_round}" }
                        }
                        div { class: "stats-kpi",
                            span { class: "stats-kpi-label", "KILLS" }
                            span { class: "stats-kpi-value stats-kpi-danger", "{kills}/{total_bosses_ever}" }
                        }
                    }

                    // ── Active Player ────────────────────────────────────────
                    div { class: "stats-current-player",
                        span { class: "stats-kpi-label", "⚔️ ACTIVE PLAYER" }
                        span { class: "stats-kpi-value stats-kpi-teal", "{current_player}" }
                    }

                    // ── Scenario Progress ────────────────────────────────────
                    div { class: "stats-section",
                        div { class: "stats-section-title", "📜 Scenario Progress" }
                        div { class: "stats-progress-bar-wrap",
                            div { class: "stats-progress-text",
                                "{completed} / {total_scenarios} completed"
                            }
                            div { class: "stats-progress-outer",
                                div {
                                    class: "stats-progress-inner",
                                    style: format!(
                                        "width: {}%",
                                        if total_scenarios > 0 {
                                            (completed * 100) / total_scenarios
                                        } else {
                                            0
                                        },
                                    ),
                                }
                            }
                        }
                        if let Some(sc) = current_scenario {
                            div { class: "stats-current-scenario",
                                span { "🗺️ " }
                                span { style: "font-weight:600;", "{sc.name}" }
                                span { class: "stats-scenario-level", " · Lv {sc.level}" }
                                if !sc.universe.is_empty() {
                                    span { class: "stats-scenario-universe", " · 🌐 {sc.universe}" }
                                }
                            }
                        }
                    }

                    // ── Heroes HP bars ───────────────────────────────────────
                    div { class: "stats-section",
                        div { class: "stats-section-title", "🧙 Heroes Status" }
                        for hero in gm.pm.active_heroes.iter() {
                            {
                                let hp_cur = hero.stats.all_stats.get(HP).map(|a| a.current).unwrap_or(0);
                                let hp_max = hero.stats.all_stats.get(HP).map(|a| a.max).unwrap_or(1);
                                let pct = if hp_max > 0 { (hp_cur.max(0) * 100) / hp_max } else { 0 };
                                let is_dead = hero.stats.is_dead().unwrap_or(false);
                                rsx! {
                                    div { class: "stats-hero-row",
                                        div { class: "stats-hero-name",
                                            if is_dead { "💀 " } else { "🟢 " }
                                            "{hero.id_name}"
                                        }
                                        div { class: "stats-hero-bar-wrap",
                                            div {
                                                class: if is_dead { "stats-hero-bar dead" } else { "stats-hero-bar" },
                                                style: format!("width:{}%", pct),
                                            }
                                            span { class: "stats-hero-hp-text", "{hp_cur}/{hp_max}" }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Separator { horizontal: true, decorative: true }

                    // ── Charts ────────────────────────────────────────────────
                    TabStats {}
                }
            }

            SheetFooter {
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Close" }
                    },
                }
            }
        }
    }
}

#[component]
fn MenuSheet(s: SheetSide, open_wnd: Signal<bool>, is_saved: Signal<bool>) -> Element {
    let _open_wnd = open_wnd; // kept for API compatibility
    let server_data = use_context::<Signal<ServerData>>();
    let server_name = crate::common::SERVER_NAME();
    let snap = server_data();
    let current_turn = snap.core_game_data.game_manager.game_state.current_turn_nb;
    let current_player = snap
        .core_game_data
        .game_manager
        .pm
        .current_player
        .id_name
        .clone();
    let players_count = snap.players_data.players_info.len();

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "☰ Menu" }
                SheetDescription { "Save your game or return to the adventure." }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "1rem",
                padding: "0 1rem",

                // Server info
                div { style: "background:var(--rpg-bg-card); border:1px solid var(--rpg-border-light); border-radius:8px; padding:10px 14px; display:grid; grid-template-columns:1fr 1fr; gap:8px;",
                    div {
                        Label {
                            html_for: "menu-srv",
                            font_size: "0.7rem",
                            color: "var(--rpg-text-muted)",
                            "SERVER"
                        }
                        div { style: "font-size:0.9rem; font-weight:600; color:var(--rpg-gold);",
                            "{server_name}"
                        }
                    }
                    div {
                        Label {
                            html_for: "menu-turn",
                            font_size: "0.7rem",
                            color: "var(--rpg-text-muted)",
                            "TURN"
                        }
                        div { style: "font-size:0.9rem; font-weight:600;", "{current_turn}" }
                    }
                    div {
                        Label {
                            html_for: "menu-player",
                            font_size: "0.7rem",
                            color: "var(--rpg-text-muted)",
                            "ACTIVE PLAYER"
                        }
                        div { style: "font-size:0.85rem; font-weight:500; color:var(--rpg-teal);",
                            "{current_player}"
                        }
                    }
                    div {
                        Label {
                            html_for: "menu-players",
                            font_size: "0.7rem",
                            color: "var(--rpg-text-muted)",
                            "PLAYERS"
                        }
                        div { style: "font-size:0.85rem; font-weight:500;", "{players_count}" }
                    }
                }

                // Save status indicator
                if is_saved() {
                    div { style: "background:#14532d; border:1px solid #22c55e; border-radius:8px; padding:8px 14px; display:flex; align-items:center; gap:8px;",
                        div { style: "font-size:0.9rem; color:#86efac;", "✅ Game saved successfully" }
                    }
                }
            }

            SheetFooter {
                SaveButton { is_saved }
                SheetClose {
                    r#as: move |attributes| rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                is_saved.set(false);
                            },
                            attributes,
                            "Close"
                        }
                    },
                }
            }
        }

    }
}

#[component]
fn LogsSheet(s: SheetSide) -> Element {
    // context
    let server_data = use_context::<Signal<ServerData>>();

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "Logs" }
                SheetDescription { "History of all game events." }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "0.5rem",
                padding: "0 1rem",
                flex: "1",
                overflow: "hidden",

                Tabs { default_value: "all".to_owned(), horizontal: true,

                    TabList {
                        TabTrigger { value: "all".to_owned(), index: 0_usize, "All" }
                        TabTrigger { value: "combat".to_owned(), index: 1_usize, "⚔ Combat" }
                        TabTrigger { value: "heal".to_owned(), index: 2_usize, "💚 Healing" }
                        TabTrigger { value: "event".to_owned(), index: 3_usize, "ℹ Events" }
                    }

                    TabContent { value: "all".to_owned(), index: 0_usize,
                        LogsList {
                            logs: server_data().core_game_data.game_manager.logs.clone(),
                            filter: "all".to_owned(),
                        }
                    }
                    TabContent { value: "combat".to_owned(), index: 1_usize,
                        LogsList {
                            logs: server_data().core_game_data.game_manager.logs.clone(),
                            filter: "combat".to_owned(),
                        }
                    }
                    TabContent { value: "heal".to_owned(), index: 2_usize,
                        LogsList {
                            logs: server_data().core_game_data.game_manager.logs.clone(),
                            filter: "heal".to_owned(),
                        }
                    }
                    TabContent { value: "event".to_owned(), index: 3_usize,
                        LogsList {
                            logs: server_data().core_game_data.game_manager.logs.clone(),
                            filter: "event".to_owned(),
                        }
                    }
                }
            }

            SheetFooter {
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Close" }
                    },
                }
            }
        }
    }
}

/// Filtered, colored list of log entries — newest first.
#[component]
fn LogsList(logs: Vec<LogData>, filter: String) -> Element {
    let filtered: Vec<&LogData> = logs
        .iter()
        .rev()
        .filter(|log| match filter.as_str() {
            "combat" => log.color == DARK_RED,
            "heal" => log.color == LIGHT_GREEN,
            "event" => log.color == LIGHT_BLUE,
            _ => true,
        })
        .collect();

    rsx! {
        ScrollArea {
            width: "100%",
            height: "calc(100vh - 18rem)",
            border: "1px solid var(--rpg-border-light)",
            border_radius: "8px",
            padding: "0.5em",
            direction: ScrollDirection::Vertical,
            tabindex: "0",
            div { class: "scroll-content",
                if filtered.is_empty() {
                    div { style: "color: var(--rpg-text-muted); text-align: center; padding: 2rem; font-size: 0.85rem;",
                        "No logs yet."
                    }
                }
                for log in filtered {
                    div { style: "padding: 4px 8px; margin: 2px 0; border-left: 3px solid {log.color}; border-radius: 0 4px 4px 0; font-size: 0.82rem; color: {log.color}; word-break: break-word;",
                        "{log.message}"
                    }
                }
            }
        }
    }
}

// ─── Scenarios Sheet ──────────────────────────────────────────────────────────

/// A sheet showing all scenarios and their completion state for the current game.
#[component]
fn ScenariosSheet(s: SheetSide) -> Element {
    let server_data = use_context::<Signal<ServerData>>();
    let snap = server_data();
    let gm = &snap.core_game_data.game_manager;
    let states = &gm.states_scenarios;

    // Sort scenarios by numeric level (not string) so level 10 comes after level 9
    let mut sorted_scenarios = gm.all_scenarios.clone();
    sorted_scenarios.sort_by_key(|s| s.level);

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "📜 Scenarios" }
                SheetDescription { "Progress through all available stages." }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "0.5rem",
                padding: "0 1rem",

                if sorted_scenarios.is_empty() {
                    div { style: "color:var(--rpg-text-muted); text-align:center; padding:2rem; font-size:0.85rem;",
                        "No scenarios loaded."
                    }
                } else {
                    div { class: "scenario-history",
                        for scenario in sorted_scenarios.iter() {
                            {
                                let state = states.get(&scenario.name).cloned().unwrap_or(ScenarioState::NotStarted);
                                let (status_text, chip_class, item_class) = match state {
                                    ScenarioState::Completed => ("✅ Completed", "scenario-chip completed", "scenario-history-item completed"),
                                    ScenarioState::InProgress => ("⚔️ In Progress", "scenario-chip in-progress", "scenario-history-item"),
                                    ScenarioState::NotStarted => ("🔒 Not Started", "scenario-chip", "scenario-history-item not-started"),
                                };
                                rsx! {
                                    div { class: item_class,
                                        span { class: "scenario-history-level", "{scenario.level}" }
                                        div { class: "scenario-history-name", "{scenario.name}" }
                                        span { class: chip_class, "{status_text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SheetFooter {
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Close" }
                    },
                }
            }
        }
    }
}

// ─── Settings Sheet ───────────────────────────────────────────────────────────

const SETTING_TOOLTIPS: &str = "show_atk_tooltips";

#[component]
fn SettingsSheet(s: SheetSide) -> Element {
    let mut show_tooltips = use_context::<Signal<bool>>();
    let mut save_msg: Signal<String> = use_signal(String::new);

    // Load saved setting on mount
    use_effect(move || {
        spawn(async move {
            if let Ok(val) = get_user_setting(SETTING_TOOLTIPS.to_string(), "true".to_string()).await {
                show_tooltips.set(val == "true");
            }
        });
    });

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "⚙️ Settings" }
                SheetDescription { "Personalise your game experience." }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "1.2rem",
                padding: "0 1rem",

                // ── Attack Tooltips ────────────────────────────────────────────
                div { class: "settings-row",
                    div { class: "settings-label-group",
                        span { class: "settings-label", "Attack Tooltips" }
                        span { class: "settings-hint",
                            "Show attack description on hover in the attack list."
                        }
                    }
                    label { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: show_tooltips(),
                            onchange: move |e| {
                                let v = e.value() == "true" || show_tooltips();
                                // toggle manually since checkbox `checked` doesn't invert
                                let new_val = !show_tooltips();
                                show_tooltips.set(new_val);
                                save_msg.set("Saving…".to_string());
                                spawn(async move {
                                    let _ = save_user_setting(
                                        SETTING_TOOLTIPS.to_string(),
                                        if new_val { "true" } else { "false" }.to_string(),
                                    )
                                    .await;
                                    save_msg.set("✅ Saved".to_string());
                                    let _ = v; // suppress warning
                                });
                            },
                        }
                        span { class: "toggle-slider" }
                    }
                }

                if !save_msg().is_empty() {
                    p { class: "settings-save-msg", "{save_msg}" }
                }
            }

            SheetFooter {
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Close" }
                    },
                }
            }
        }
    }
}

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_primitives::scroll_area::ScrollDirection;
use lib_rpg::{
    character_mod::loot::LootType,
    common::{
        constants::stats_const::HP,
        log_data::{
            LogData,
            const_colors::{DARK_RED, LIGHT_BLUE, LIGHT_GREEN},
        },
    },
    server::{scenario::ScenarioState, server_manager::ServerData},
    shop::sell_price,
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
    Store,
}

#[component]
pub fn GameSheets() -> Element {
    let mut open = use_signal(|| false);
    let mut sheet_kind: Signal<SheetKind> = use_signal(|| SheetKind::Menu);
    let mut is_saved: Signal<bool> = use_signal(|| false);
    let server_data = use_context::<Signal<ServerData>>();
    let shop_enabled = use_context::<crate::common::CtxShopEnabled>().0;

    // Load shop_enabled from DB on mount so the button reflects the saved
    // setting without the user having to open Settings first.
    use_effect(move || {
        spawn(async move {
            if let Ok(val) =
                get_user_setting("shop_enabled".to_owned(), "false".to_owned()).await
            {
                shop_enabled.set(val == "true");
            }
        });
    });

    let open_sheet = move |kind: SheetKind| {
        move |_| {
            sheet_kind.set(kind.clone());
            open.set(true);
        }
    };
    if !open() {
        is_saved.set(false);
    }

    // Dead heroes can still be granted loot, but they should not light up the
    // "new equipment" notification — only living heroes the player can act on do.
    let has_new_equipment = server_data()
        .core_game_data
        .game_manager
        .pm
        .active_heroes
        .iter()
        .any(|h| !h.stats.is_dead().unwrap_or(false) && h.inventory.has_unseen_equipment());

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
                position: "relative",
                "Inventory"
                if has_new_equipment {
                    span {
                        class: "equip-tab-new-badge",
                        style: "position:absolute;top:2px;right:2px;",
                        title: "New equipment!",
                    }
                }
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
            Button {
                variant: ButtonVariant::Outline,
                disabled: !shop_enabled(),
                title: if shop_enabled() { "" } else { "Enable \"Shop During Scenario\" in ⚙️ Settings" },
                onclick: open_sheet(SheetKind::Store),
                "🛒 Store"
            }
        }
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            match sheet_kind() {
                SheetKind::Inventory => rsx! {
                    InventorySheet { s: SheetSide::Right }
                },
                SheetKind::Stats => rsx! {
                    GameStatsSheet { s: SheetSide::Left }
                },
                SheetKind::Menu => rsx! {
                    MenuSheet { s: SheetSide::Top, open_wnd: open, is_saved }
                },
                SheetKind::Logs => rsx! {
                    LogsSheet { s: SheetSide::Bottom }
                },
                SheetKind::Scenarios => rsx! {
                    ScenariosSheet { s: SheetSide::Right }
                },
                SheetKind::Settings => rsx! {
                    SettingsSheet { s: SheetSide::Left }
                },
                SheetKind::Store => rsx! {
                    StoreSheet { s: SheetSide::Right }
                },
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
    let gm = &server_data_snap.core_game_data.game_manager;
    let is_single_player = server_data_snap.core_game_data.is_single_player;

    // In single-player mode show a tab per hero; otherwise only show the logged-in hero.
    let heroes_to_show: Vec<lib_rpg::character_mod::character::Character> = if is_single_player {
        gm.pm.active_heroes.clone()
    } else {
        let Some(character_name) = server_data_snap
            .players_data
            .get_first_character_name(&local_login_name_session())
        else {
            return rsx! {};
        };
        match gm.pm.get_active_hero_character(&character_name) {
            Some(c) => vec![c.clone()],
            None => return rsx! {},
        }
    };

    // Tab state – index of currently visible hero tab
    let mut active_tab: Signal<usize> = use_signal(|| 0);
    let active_tab_idx = active_tab().min(heroes_to_show.len().saturating_sub(1));
    let character = heroes_to_show
        .get(active_tab_idx)
        .cloned()
        .unwrap_or_default();

    // BTreeMap — all stats sorted
    let ordered_stats: std::collections::BTreeMap<
        String,
        lib_rpg::character_mod::stats::Attribute,
    > = character.stats.all_stats.clone().into_iter().collect();

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

                // Hero selector tabs — only shown in single-player when there are multiple heroes
                if heroes_to_show.len() > 1 {
                    div { display: "flex", gap: "0.4rem", flex_wrap: "wrap",
                        for (i, hero) in heroes_to_show.iter().enumerate() {
                            button {
                                class: if i == active_tab_idx { "inv-tab inv-tab--active" } else { "inv-tab" },
                                onclick: move |_| active_tab.set(i),
                                position: "relative",
                                "{hero.db_full_name}"
                                if !hero.stats.is_dead().unwrap_or(false) && hero.inventory.has_unseen_equipment() {
                                    span {
                                        class: "equip-tab-new-badge",
                                        style: "position:absolute;top:2px;right:2px;",
                                        title: "New equipment for {hero.db_full_name}!",
                                    }
                                }
                            }
                        }
                    }
                }

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

                // Equipment tabs.
                // Key by hero id so the component fully remounts when the active hero
                // changes: this resets the selected category tab and re-runs the
                // "mark category seen" effect for the newly shown hero. Without the
                // key, the effect's captured hero id / category list stays stale and
                // the "new equipment" badges never clear for other heroes.
                TabEquipment { key: "{character.id_name}", c: character.clone() }
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

    // Kill count: accumulated from past scenarios plus currently-dead bosses in active_bosses.
    // all_bosses are templates that never take damage, so we must NOT count them.
    let kills = gm.game_state.accumulated_kills
        + gm.pm
            .active_bosses
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
                            span { class: "stats-kpi-value stats-kpi-danger",
                                "{kills}/{total_bosses_ever}"
                            }
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
                                        (completed * 100).checked_div(total_scenarios).unwrap_or(0),
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
                                let pct = (hp_cur * 100).checked_div(hp_max).unwrap_or(0);
                                let is_dead = hero.stats.is_dead().unwrap_or(false);
                                rsx! {
                                    div { class: "stats-hero-row",
                                        div { class: "stats-hero-name",
                                            if is_dead {
                                                "💀 "
                                            } else {
                                                "🟢 "
                                            }
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
                    {
                        let msg = log.message.replace('\n', "<br/>");
                        rsx! {
                            div {
                                style: "padding: 4px 8px; margin: 2px 0; border-left: 3px solid {log.color}; border-radius: 0 4px 4px 0; font-size: 0.82rem; color: {log.color}; word-break: break-word;",
                                dangerous_inner_html: "{msg}",
                            }
                        }
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
                                let state = states
                                    .get(&scenario.name)
                                    .cloned()
                                    .unwrap_or(ScenarioState::NotStarted);
                                let (status_text, chip_class, item_class) = match state {
                                    ScenarioState::Completed => {
                                        (
                                            "✅ Completed",
                                            "scenario-chip completed",
                                            "scenario-history-item completed",
                                        )
                                    }
                                    ScenarioState::InProgress => {
                                        (
                                            "⚔️ In Progress",
                                            "scenario-chip in-progress",
                                            "scenario-history-item",
                                        )
                                    }
                                    ScenarioState::NotStarted => {
                                        (
                                            "🔒 Not Started",
                                            "scenario-chip",
                                            "scenario-history-item not-started",
                                        )
                                    }
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

// ─── Store Sheet ─────────────────────────────────────────────────────────────

fn rank_color(rank: &lib_rpg::character_mod::rank::Rank) -> &'static str {
    match rank {
        lib_rpg::character_mod::rank::Rank::Common => "var(--rpg-text-muted)",
        lib_rpg::character_mod::rank::Rank::Intermediate => "#4a9eff",
        lib_rpg::character_mod::rank::Rank::Advanced => "#a855f7",
    }
}

fn rank_label(rank: &lib_rpg::character_mod::rank::Rank) -> &'static str {
    match rank {
        lib_rpg::character_mod::rank::Rank::Common => "Common",
        lib_rpg::character_mod::rank::Rank::Intermediate => "Intermediate",
        lib_rpg::character_mod::rank::Rank::Advanced => "Advanced",
    }
}

/// The full Store sheet — browse items for sale, buy and sell.
/// Available at end-of-scenario. Accessible from both `GameSheets` (disabled during
/// combat) and the scenario-end screen.
#[component]
pub fn StoreSheet(s: SheetSide) -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let server_data_snap = server_data();
    let gm = &server_data_snap.core_game_data.game_manager;
    let is_single_player = server_data_snap.core_game_data.is_single_player;

    let shop_catalog = server_data_snap.core_game_data.shop_catalog.clone();

    let heroes_to_show: Vec<lib_rpg::character_mod::character::Character> = if is_single_player {
        gm.pm.active_heroes.clone()
    } else {
        let Some(character_name) = server_data_snap
            .players_data
            .get_first_character_name(&local_login_name_session())
        else {
            return rsx! {};
        };
        match gm.pm.get_active_hero_character(&character_name) {
            Some(c) => vec![c.clone()],
            None => return rsx! {},
        }
    };

    let mut active_tab: Signal<usize> = use_signal(|| 0);
    let active_tab_idx = active_tab().min(heroes_to_show.len().saturating_sub(1));
    let character = heroes_to_show
        .get(active_tab_idx)
        .cloned()
        .unwrap_or_default();

    let char_id = character.id_name.clone();
    let gold = character.inventory.money;
    // 0 = Shop, 1 = Bag  (signal-driven; never unmounts children)
    let mut main_tab: Signal<u8> = use_signal(|| 0);
    // 0 = Equipment, 1 = Consumables
    let mut shop_sub_tab: Signal<u8> = use_signal(|| 0);

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "🛒 Store — {character.db_full_name}" }
                SheetDescription {
                    "Level {character.level} · "
                    span {
                        style: "color: var(--rpg-gold, #c9a227); font-weight: 700;",
                        "💰 {gold} gold"
                    }
                }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "1rem",
                padding: "0 1rem",

                // Hero selector (single-player with multiple heroes)
                if heroes_to_show.len() > 1 {
                    div { display: "flex", gap: "0.4rem", flex_wrap: "wrap",
                        for (i, hero) in heroes_to_show.iter().enumerate() {
                            button {
                                class: if i == active_tab_idx { "inv-tab inv-tab--active" } else { "inv-tab" },
                                onclick: move |_| active_tab.set(i),
                                "{hero.db_full_name}"
                            }
                        }
                    }
                }

                // Main tab row: Shop | Bag
                div { display: "flex", gap: "0.4rem",
                    button {
                        class: if main_tab() == 0 { "inv-tab inv-tab--active" } else { "inv-tab" },
                        onclick: move |_| main_tab.set(0),
                        "🏪 Shop"
                    }
                    button {
                        class: if main_tab() == 1 { "inv-tab inv-tab--active" } else { "inv-tab" },
                        onclick: move |_| main_tab.set(1),
                        "🎒 Bag"
                    }
                }

                // ── Shop panel (always mounted) ────────────────────────────
                div { display: if main_tab() == 0 { "flex" } else { "none" },
                    flex_direction: "column",
                    gap: "0.5rem",

                    // Sub-tab buttons: Equipment | Consumables
                    div { display: "flex", gap: "0.4rem",
                        button {
                            class: if shop_sub_tab() == 0 { "inv-tab inv-tab--active" } else { "inv-tab" },
                            onclick: move |_| shop_sub_tab.set(0),
                            "⚔️ Equipment"
                        }
                        button {
                            class: if shop_sub_tab() == 1 { "inv-tab inv-tab--active" } else { "inv-tab" },
                            onclick: move |_| shop_sub_tab.set(1),
                            "💊 Consumables"
                        }
                    }

                            // Equipment sub-panel — always mounted, hidden via display
                            div { display: if shop_sub_tab() == 0 { "block" } else { "none" },
                                ScrollArea {
                                    width: "100%",
                                    height: "calc(100vh - 24rem)",
                                    direction: ScrollDirection::Vertical,
                                    div {
                                        display: "flex",
                                        flex_direction: "column",
                                        gap: "0.5rem",
                                        padding: "0.5rem 0",
                                        for item in shop_catalog.iter().filter(|i| i.kind == LootType::Equipment) {
                                            {
                                                let item = item.clone();
                                                let char_id_clone = char_id.clone();

                                                let can_afford = gold >= item.price;
                                                let bag_count = character.inventory.equipments
                                                    .values()
                                                    .flatten()
                                                    .filter(|e| e.unique_name == item.name)
                                                    .count();
                                                let category_label = item.category.as_ref()
                                                    .map(|c| c.to_string())
                                                    .unwrap_or_default();
                                                let rank_col = rank_color(&item.rank);
                                                let rank_lbl = rank_label(&item.rank);
                                                rsx! {
                                                    div {
                                                        style: "border:1px solid var(--rpg-border);border-radius:8px;padding:0.75rem;display:flex;flex-direction:column;gap:0.4rem;",
                                                        div {
                                                            style: "display:flex;justify-content:space-between;align-items:center;",
                                                            span {
                                                                style: "font-weight:700;font-size:0.9rem;",
                                                                "{item.name}"
                                                            }
                                                            span {
                                                                style: "font-size:0.72rem;font-weight:600;color:{rank_col};border:1px solid {rank_col};border-radius:4px;padding:1px 6px;",
                                                                "{rank_lbl}"
                                                            }
                                                        }
                                                        span {
                                                            style: "font-size:0.75rem;color:var(--rpg-text-muted);",
                                                            "Slot: {category_label}"
                                                        }
                                                        span {
                                                            style: "font-size:0.78rem;color:var(--rpg-text-secondary,var(--rpg-text-muted));",
                                                            "{item.description}"
                                                        }
                                                        div {
                                                            style: "display:flex;align-items:center;justify-content:space-between;margin-top:0.25rem;",
                                                            span {
                                                                style: "color:var(--rpg-gold,#c9a227);font-weight:600;font-size:0.85rem;",
                                                                "💰 {item.price} gold"
                                                                if bag_count > 0 {
                                                                    span {
                                                                        style: "margin-left:0.4rem;font-size:0.75rem;color:var(--rpg-text-muted);",
                                                                        "(×{bag_count} in bag)"
                                                                    }
                                                                }
                                                            }
                                                            Button {
                                                                variant: if can_afford { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                                                disabled: !can_afford,
                                                                onclick: {
                                                                    let item_name = item.name.clone();
                                                                    let cid = char_id_clone.clone();
                                                                    move |_| {
                                                                        let item_name = item_name.clone();
                                                                        let cid = cid.clone();
                                                                        async move {
                                                                            let _ = socket.send(ClientEvent::BuyItem(
                                                                                crate::common::SERVER_NAME(),
                                                                                cid,
                                                                                item_name,
                                                                                "Equipment".to_owned(),
                                                                            )).await;
                                                                        }
                                                                    }
                                                                },
                                                                if can_afford { "Buy" } else { "No gold" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Consumables sub-panel — always mounted, hidden via display
                            div { display: if shop_sub_tab() == 1 { "block" } else { "none" },
                                ScrollArea {
                                    width: "100%",
                                    height: "calc(100vh - 24rem)",
                                    direction: ScrollDirection::Vertical,
                                    div {
                                        display: "flex",
                                        flex_direction: "column",
                                        gap: "0.5rem",
                                        padding: "0.5rem 0",
                                        for item in shop_catalog.iter().filter(|i| i.kind == LootType::Consumable) {
                                            {
                                                let item = item.clone();
                                                let char_id_clone = char_id.clone();

                                                let can_afford = gold >= item.price;
                                                let bag_count = character.inventory.consumables
                                                    .iter()
                                                    .filter(|c| c.name == item.name)
                                                    .count();
                                                let rank_col = rank_color(&item.rank);
                                                let rank_lbl = rank_label(&item.rank);
                                                rsx! {
                                                    div {
                                                        style: "border:1px solid var(--rpg-border);border-radius:8px;padding:0.75rem;display:flex;flex-direction:column;gap:0.4rem;",
                                                        div {
                                                            style: "display:flex;justify-content:space-between;align-items:center;",
                                                            span {
                                                                style: "font-weight:700;font-size:0.9rem;",
                                                                "{item.name}"
                                                            }
                                                            span {
                                                                style: "font-size:0.72rem;font-weight:600;color:{rank_col};border:1px solid {rank_col};border-radius:4px;padding:1px 6px;",
                                                                "{rank_lbl}"
                                                            }
                                                        }
                                                        span {
                                                            style: "font-size:0.78rem;color:var(--rpg-text-secondary,var(--rpg-text-muted));",
                                                            "{item.description}"
                                                        }
                                                        div {
                                                            style: "display:flex;align-items:center;justify-content:space-between;margin-top:0.25rem;",
                                                            span {
                                                                style: "color:var(--rpg-gold,#c9a227);font-weight:600;font-size:0.85rem;",
                                                                "💰 {item.price} gold"
                                                                if bag_count > 0 {
                                                                    span {
                                                                        style: "margin-left:0.4rem;font-size:0.75rem;color:var(--rpg-text-muted);",
                                                                        "(×{bag_count} in bag)"
                                                                    }
                                                                }
                                                            }
                                                            Button {
                                                                variant: if can_afford { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                                                disabled: !can_afford,
                                                                onclick: {
                                                                    let item_name = item.name.clone();
                                                                    let cid = char_id_clone.clone();
                                                                    move |_| {
                                                                        let item_name = item_name.clone();
                                                                        let cid = cid.clone();
                                                                        async move {
                                                                            let _ = socket.send(ClientEvent::BuyItem(
                                                                                crate::common::SERVER_NAME(),
                                                                                cid,
                                                                                item_name,
                                                                                "Consumable".to_owned(),
                                                                            )).await;
                                                                        }
                                                                    }
                                                                },
                                                                if can_afford { "Buy" } else { "No gold" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                // ── Bag panel (always mounted) ─────────────────────────────
                div { display: if main_tab() == 1 { "flex" } else { "none" },
                    flex_direction: "column",
                    gap: "0.75rem",

                            {
                                // Collect unequipped equipment items
                                let unequipped: Vec<(String, String)> = character.inventory.equipments
                                    .iter()
                                    .flat_map(|(cat, items)| {
                                        items.iter()
                                            .filter(|e| !e.is_equipped)
                                            .map(|e| (e.unique_name.clone(), cat.to_string()))
                                            .collect::<Vec<_>>()
                                    })
                                    .collect();

                                let bag_consumables = character.inventory.consumables.clone();

                                let is_empty = unequipped.is_empty() && bag_consumables.is_empty();

                                rsx! {
                                    ScrollArea {
                                        width: "100%",
                                        height: "calc(100vh - 18rem)",
                                        direction: ScrollDirection::Vertical,

                                        if is_empty {
                                            div {
                                                style: "color:var(--rpg-text-muted);text-align:center;padding:2rem;font-size:0.85rem;",
                                                "Your bag is empty."
                                            }
                                        } else {
                                            div {
                                                display: "flex",
                                                flex_direction: "column",
                                                gap: "0.5rem",
                                                padding: "0.25rem 0",

                                                // Equipment section
                                                if !unequipped.is_empty() {
                                                    span {
                                                        style: "font-size:0.8rem;font-weight:700;color:var(--rpg-text-muted);text-transform:uppercase;letter-spacing:0.05em;padding:0.25rem 0;",
                                                        "⚔️ Equipment"
                                                    }
                                                    for (unique_name, cat_label) in unequipped.iter() {
                                                        {
                                                            let unique_name = unique_name.clone();
                                                            let cat_label = cat_label.clone();
                                                            let char_id_clone = char_id.clone();

                                                            let refund = shop_catalog.iter()
                                                                .find(|i| i.name == unique_name)
                                                                .map(|i| sell_price(i.price))
                                                                .unwrap_or(0);
                                                            rsx! {
                                                                div {
                                                                    style: "border:1px solid var(--rpg-border);border-radius:8px;padding:0.6rem 0.75rem;display:flex;align-items:center;justify-content:space-between;gap:0.5rem;",
                                                                    div {
                                                                        display: "flex",
                                                                        flex_direction: "column",
                                                                        span {
                                                                            style: "font-weight:600;font-size:0.85rem;",
                                                                            "{unique_name}"
                                                                        }
                                                                        span {
                                                                            style: "font-size:0.75rem;color:var(--rpg-text-muted);",
                                                                            "Slot: {cat_label}"
                                                                        }
                                                                    }
                                                                    div {
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        gap: "0.5rem",
                                                                        span {
                                                                            style: "color:var(--rpg-gold,#c9a227);font-size:0.8rem;font-weight:600;white-space:nowrap;",
                                                                            "💰 {refund} gold"
                                                                        }
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            onclick: {
                                                                                let name = unique_name.clone();
                                                                                let cid = char_id_clone.clone();
                                                                                move |_| {
                                                                                    let name = name.clone();
                                                                                    let cid = cid.clone();
                                                                                    async move {
                                                                                        let _ = socket
                                                                                            .send(ClientEvent::SellItem(
                                                                                                crate::common::SERVER_NAME(),
                                                                                                cid,
                                                                                                name,
                                                                                                "Equipment".to_owned(),
                                                                                            ))
                                                                                            .await;
                                                                                    }
                                                                                }
                                                                            },
                                                                            "Sell"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                // Consumables section
                                                if !bag_consumables.is_empty() {
                                                    span {
                                                        style: "font-size:0.8rem;font-weight:700;color:var(--rpg-text-muted);text-transform:uppercase;letter-spacing:0.05em;padding:0.25rem 0;margin-top:0.25rem;",
                                                        "💊 Consumables"
                                                    }
                                                    for consumable in bag_consumables.iter() {
                                                        {
                                                            let consumable_name = consumable.name.clone();
                                                            let char_id_clone = char_id.clone();

                                                            let refund = shop_catalog.iter()
                                                                .find(|i| i.name == consumable_name)
                                                                .map(|i| sell_price(i.price))
                                                                .unwrap_or(0);
                                                            let rank_col = rank_color(&consumable.rank);
                                                            let rank_lbl = rank_label(&consumable.rank);
                                                            rsx! {
                                                                div {
                                                                    style: "border:1px solid var(--rpg-border);border-radius:8px;padding:0.6rem 0.75rem;display:flex;align-items:center;justify-content:space-between;gap:0.5rem;",
                                                                    div {
                                                                        display: "flex",
                                                                        flex_direction: "column",
                                                                        gap: "0.15rem",
                                                                        span {
                                                                            style: "font-weight:600;font-size:0.85rem;",
                                                                            "{consumable_name}"
                                                                        }
                                                                        span {
                                                                            style: "font-size:0.72rem;font-weight:600;color:{rank_col};",
                                                                            "{rank_lbl}"
                                                                        }
                                                                    }
                                                                    div {
                                                                        display: "flex",
                                                                        align_items: "center",
                                                                        gap: "0.5rem",
                                                                        span {
                                                                            style: "color:var(--rpg-gold,#c9a227);font-size:0.8rem;font-weight:600;white-space:nowrap;",
                                                                            "💰 {refund} gold"
                                                                        }
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            onclick: {
                                                                                let name = consumable_name.clone();
                                                                                let cid = char_id_clone.clone();
                                                                                move |_| {
                                                                                    let name = name.clone();
                                                                                    let cid = cid.clone();
                                                                                    async move {
                                                                                        let _ = socket
                                                                                            .send(ClientEvent::SellItem(
                                                                                                crate::common::SERVER_NAME(),
                                                                                                cid,
                                                                                                name,
                                                                                                "Consumable".to_owned(),
                                                                                            ))
                                                                                            .await;
                                                                                    }
                                                                                }
                                                                            },
                                                                            "Sell"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
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
const SETTING_BOSS_ENERGY: &str = "show_boss_energy";
const SETTING_HERO_AGGRO: &str = "show_hero_aggro";
const SETTING_BOSS_HP: &str = "show_boss_hp";
const SETTING_AUTO_SAVE: &str = "auto_save_on_scenario";
const SETTING_SHOP_ENABLED: &str = "shop_enabled";

#[component]
fn SettingsSheet(s: SheetSide) -> Element {
    let mut show_atk_tooltips = use_context::<crate::common::CtxShowAtkTooltips>().0;
    let mut show_boss_energy = use_context::<crate::common::CtxShowBossEnergy>().0;
    let mut show_hero_aggro = use_context::<crate::common::CtxShowHeroAggro>().0;
    let mut show_boss_hp = use_context::<crate::common::CtxShowBossHp>().0;
    let mut auto_save_scenario = use_context::<crate::common::CtxAutoSaveScenario>().0;
    let mut shop_enabled = use_context::<crate::common::CtxShopEnabled>().0;
    let mut save_msg: Signal<String> = use_signal(String::new);

    // Load saved settings on mount
    use_effect(move || {
        spawn(async move {
            if let Ok(val) = get_user_setting(SETTING_TOOLTIPS.to_string(), "true".to_owned()).await
            {
                show_atk_tooltips.set(val == "true");
            }
            if let Ok(val) =
                get_user_setting(SETTING_BOSS_ENERGY.to_string(), "true".to_owned()).await
            {
                show_boss_energy.set(val == "true");
            }
            if let Ok(val) =
                get_user_setting(SETTING_HERO_AGGRO.to_string(), "true".to_owned()).await
            {
                show_hero_aggro.set(val == "true");
            }
            if let Ok(val) = get_user_setting(SETTING_BOSS_HP.to_string(), "true".to_owned()).await
            {
                show_boss_hp.set(val == "true");
            }
            if let Ok(val) =
                get_user_setting(SETTING_AUTO_SAVE.to_string(), "true".to_owned()).await
            {
                auto_save_scenario.set(val == "true");
            }
            if let Ok(val) =
                get_user_setting(SETTING_SHOP_ENABLED.to_string(), "false".to_owned()).await
            {
                shop_enabled.set(val == "true");
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
                            checked: show_atk_tooltips(),
                            onchange: move |e| {
                                let v = e.value() == "true" || show_atk_tooltips();
                                let new_val = !show_atk_tooltips();
                                show_atk_tooltips.set(new_val);
                                save_msg.set("Saving…".to_owned());
                                spawn(async move {
                                    let _ = save_user_setting(
                                            SETTING_TOOLTIPS.to_string(),
                                            if new_val { "true" } else { "false" }.to_string(),
                                        )
                                        .await;
                                    save_msg.set("✅ Saved".to_owned());
                                    let _ = v;
                                });
                            },
                        }
                        span { class: "toggle-slider" }
                    }
                }

                // ── Boss Energy Bars ───────────────────────────────────────────
                div { class: "settings-row",
                    div { class: "settings-label-group",
                        span { class: "settings-label", "Boss Energy Bars" }
                        span { class: "settings-hint", "Show mana/vigor/berserk bars for bosses." }
                    }
                    label { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: show_boss_energy(),
                            onchange: move |_| {
                                let new_val = !show_boss_energy();
                                show_boss_energy.set(new_val);
                                save_msg.set("Saving…".to_owned());
                                spawn(async move {
                                    let _ = save_user_setting(
                                            SETTING_BOSS_ENERGY.to_string(),
                                            if new_val { "true" } else { "false" }.to_string(),
                                        )
                                        .await;
                                    save_msg.set("✅ Saved".to_owned());
                                });
                            },
                        }
                        span { class: "toggle-slider" }
                    }
                }

                // ── Hero Aggro ─────────────────────────────────────────────────
                div { class: "settings-row",
                    div { class: "settings-label-group",
                        span { class: "settings-label", "Hero Aggro" }
                        span { class: "settings-hint", "Show aggro value on the hero panel header." }
                    }
                    label { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: show_hero_aggro(),
                            onchange: move |_| {
                                let new_val = !show_hero_aggro();
                                show_hero_aggro.set(new_val);
                                save_msg.set("Saving…".to_owned());
                                spawn(async move {
                                    let _ = save_user_setting(
                                            SETTING_HERO_AGGRO.to_string(),
                                            if new_val { "true" } else { "false" }.to_string(),
                                        )
                                        .await;
                                    save_msg.set("✅ Saved".to_owned());
                                });
                            },
                        }
                        span { class: "toggle-slider" }
                    }
                }

                // ── Boss HP Bar ────────────────────────────────────────────────
                div { class: "settings-row",
                    div { class: "settings-label-group",
                        span { class: "settings-label", "Boss HP Bar" }
                        span { class: "settings-hint",
                            "Show the HP bar on boss panels (hidden if you want mystery)."
                        }
                    }
                    label { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: show_boss_hp(),
                            onchange: move |_| {
                                let new_val = !show_boss_hp();
                                show_boss_hp.set(new_val);
                                save_msg.set("Saving…".to_owned());
                                spawn(async move {
                                    let _ = save_user_setting(
                                            SETTING_BOSS_HP.to_string(),
                                            if new_val { "true" } else { "false" }.to_string(),
                                        )
                                        .await;
                                    save_msg.set("✅ Saved".to_owned());
                                });
                            },
                        }
                        span { class: "toggle-slider" }
                    }
                }

                // ── Auto-save on scenario start ────────────────────────────────
                div { class: "settings-row",
                    div { class: "settings-label-group",
                        span { class: "settings-label", "Auto-save on Scenario" }
                        span { class: "settings-hint",
                            "Automatically save at the start of each new scenario."
                        }
                    }
                    label { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: auto_save_scenario(),
                            onchange: move |_| {
                                let new_val = !auto_save_scenario();
                                auto_save_scenario.set(new_val);
                                save_msg.set("Saving…".to_owned());
                                spawn(async move {
                                    let _ = save_user_setting(
                                            SETTING_AUTO_SAVE.to_string(),
                                            if new_val { "true" } else { "false" }.to_string(),
                                        )
                                        .await;
                                    save_msg.set("✅ Saved".to_owned());
                                });
                            },
                        }
                        span { class: "toggle-slider" }
                    }
                }

                // ── Shop During Scenario ───────────────────────────────────────
                div { class: "settings-row",
                    div { class: "settings-label-group",
                        span { class: "settings-label", "Shop During Scenario" }
                        span { class: "settings-hint",
                            "Allow opening the Store during an active scenario."
                        }
                    }
                    label { class: "toggle-switch",
                        input {
                            r#type: "checkbox",
                            checked: shop_enabled(),
                            onchange: move |_| {
                                let new_val = !shop_enabled();
                                shop_enabled.set(new_val);
                                save_msg.set("Saving…".to_owned());
                                spawn(async move {
                                    let _ = save_user_setting(
                                            SETTING_SHOP_ENABLED.to_string(),
                                            if new_val { "true" } else { "false" }.to_string(),
                                        )
                                        .await;
                                    save_msg.set("✅ Saved".to_owned());
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

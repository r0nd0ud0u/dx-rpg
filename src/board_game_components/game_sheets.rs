use std::collections::BTreeMap;

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_primitives::scroll_area::ScrollDirection;
use lib_rpg::{
    character_mod::{character::Character, stats::Attribute},
    server::server_manager::ServerData,
};

use crate::{
    components::{
        button::{Button, ButtonVariant},
        label::Label,
        scroll_area::ScrollArea,
        separator::Separator,
        sheet::{
            Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide,
            SheetTitle,
        },
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

#[component]
pub fn GameSheets() -> Element {
    let mut open = use_signal(|| false);
    let mut side = use_signal(|| SheetSide::Right);
    let mut is_saved: Signal<bool> = use_signal(|| false);

    let open_sheet = move |s: SheetSide| {
        move |_| {
            side.set(s);
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
                onclick: open_sheet(SheetSide::Top),
                "Menu"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetSide::Right),
                "Inventory"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetSide::Bottom),
                "Logs"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetSide::Left),
                "Game stats"
            }
        }
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            match side() {
                SheetSide::Right => {
                    InventorySheet(InventorySheetProps {
                        s: SheetSide::Right,
                    })
                }
                SheetSide::Left => {
                    GameStatsSheet(GameStatsSheetProps {
                        s: SheetSide::Left,
                    })
                }
                SheetSide::Top => {
                    MenuSheet(MenuSheetProps {
                        s: SheetSide::Top,
                        open_wnd: open,
                        is_saved,
                    })
                }
                SheetSide::Bottom => {
                    LogsSheet(LogsSheetProps {
                        s: SheetSide::Bottom,
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
    // context
    let server_data = use_context::<Signal<ServerData>>();

    let snap = server_data();
    let game_state = &snap.core_game_data.game_manager.game_state;
    let current_player = snap
        .core_game_data
        .game_manager
        .pm
        .current_player
        .id_name
        .clone();
    let current_round = game_state.current_round;
    let total_in_round = game_state.order_to_play.len();
    let current_turn = game_state.current_turn_nb;

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "📊 Game Stats" }
                SheetDescription { "Evolution of the current game." }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "1rem",
                padding: "0 1rem",

                // Turn / Round info block
                div {
                    display: "grid",
                    grid_template_columns: "1fr 1fr",
                    gap: "0.5rem",
                    // Turn
                    div { style: "background:var(--rpg-bg-card); border:1px solid var(--rpg-border-light); border-radius:8px; padding:8px 12px; text-align:center;",
                        Label {
                            html_for: "stat",
                            font_size: "0.7rem",
                            color: "var(--rpg-text-muted)",
                            "TURN"
                        }
                        div { style: "font-size:1.4rem; font-weight:700; color:var(--rpg-gold);",
                            "{current_turn}"
                        }
                    }
                    // Round
                    div { style: "background:var(--rpg-bg-card); border:1px solid var(--rpg-border-light); border-radius:8px; padding:8px 12px; text-align:center;",
                        Label {
                            html_for: "stat",
                            font_size: "0.7rem",
                            color: "var(--rpg-text-muted)",
                            "ROUND"
                        }
                        div { style: "font-size:1.4rem; font-weight:700; color:var(--rpg-gold);",
                            "{current_round}/{total_in_round}"
                        }
                    }
                }

                // Current player
                div { style: "background:var(--rpg-bg-card); border:1px solid var(--rpg-teal); border-radius:8px; padding:8px 12px;",
                    Label {
                        html_for: "stat",
                        font_size: "0.7rem",
                        color: "var(--rpg-text-muted)",
                        "⚔️ ACTIVE PLAYER"
                    }
                    div { style: "font-size:0.95rem; font-weight:600; color:var(--rpg-teal);",
                        "{current_player}"
                    }
                }

                Separator { horizontal: true, decorative: true }

                // Stats charts
                TabStats {}
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
                SheetDescription { "Watch the last logs here." }
            }

            div {
                display: "flex",
                flex_direction: "column",
                gap: "0.5rem",
                padding: "0 1rem",
                flex: "1",
                ScrollArea {
                    width: "100%",
                    height: "calc(100% - 2rem)",
                    border: "1px solid var(--rpg-border-light)",
                    border_radius: "8px",
                    padding: "0.5em 1em",
                    direction: ScrollDirection::Vertical,
                    tabindex: "0",
                    div { class: "scroll-content",
                        for log in server_data().core_game_data.game_manager.logs.iter() {
                            div { style: "padding: 2px 0; border-bottom: 1px solid var(--rpg-border);",
                                Label {
                                    color: "{log.color}",
                                    html_for: "sheet-log",
                                    font_size: "0.82rem",
                                    "{log.message}"
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

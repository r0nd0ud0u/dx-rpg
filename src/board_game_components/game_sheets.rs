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
    widgets::tab::TabEquipment,
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
    let character_name = server_data_snap
        .players_data
        .players_info
        .get(&local_login_name_session())
        .and_then(|info| info.character_id_names.first())
        .cloned()
        .unwrap_or_else(|| "No character selected".to_string());
    let character = match server_data_snap
        .core_game_data
        .game_manager
        .pm
        .get_active_hero_character(&character_name)
    {
        Some(c) => c.clone(),
        None => Character::default(),
    };

    // BTreeMap
    let ordered_stats: BTreeMap<String, Attribute> =
        character.stats.all_stats.clone().into_iter().collect();
    let segment_len = ordered_stats.len() / 3 + 1;
    let stats_1 = ordered_stats
        .iter()
        .take(segment_len)
        .map(|(k, v)| (k, v.clone()));
    let stats_2 = ordered_stats
        .iter()
        .skip(segment_len)
        .take(segment_len)
        .map(|(k, v)| (k, v.clone()));
    let stats_3 = ordered_stats
        .iter()
        .skip(2 * segment_len)
        .take(segment_len)
        .map(|(k, v)| (k, v.clone()));
    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "Inventory" }
                SheetDescription { "Update your equipment here." }
            }

            div {
                display: "grid",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",

                div {
                    display: "grid",
                    grid_template_columns: "max-content max-content max-content max-content max-content",
                    column_gap: "80px",
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: "0.75rem",
                        for (k , v) in stats_1 {
                            div { style: "display: flex; direction: row; gap: 0.5rem;",
                                Label {
                                    html_for: "sheet-demo-name",
                                    width: "150px",
                                    "{k}:"
                                }
                                Label { html_for: "sheet-demo-name", "{v.max}" }
                            }
                            Separator {
                                style: "margin: 5px 0;",
                                horizontal: true,
                                decorative: true,
                            }
                        }
                    }
                    Separator { horizontal: false, decorative: true }
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: "0.75rem",
                        for (k , v) in stats_2 {
                            div { style: "display: flex; direction: row; gap: 0.5rem;",
                                Label {
                                    html_for: "sheet-demo-name",
                                    width: "150px",
                                    "{k}:"
                                }
                                Label { html_for: "sheet-demo-name", "{v.max}" }
                            }
                            Separator {
                                style: "margin: 5px 0;",
                                horizontal: true,
                                decorative: true,
                            }
                        }
                    }
                    Separator { horizontal: false, decorative: true }
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: "0.75rem",
                        for (k , v) in stats_3 {
                            div { style: "display: flex; direction: row; gap: 0.5rem;",
                                Label {
                                    html_for: "sheet-demo-name",
                                    width: "150px",
                                    "{k}:"
                                }
                                Label { html_for: "sheet-demo-name", "{v.max}" }
                            }
                            Separator {
                                style: "margin: 5px 0;",
                                horizontal: true,
                                decorative: true,
                            }
                        }
                    }
                }

                TabEquipment { c: character.clone() }
            }

            SheetFooter {
                Button { "Save changes" }
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Cancel" }
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

    let current_round = format!(
        "Current round: {}",
        server_data()
            .core_game_data
            .game_manager
            .game_state
            .current_round
    );
    let current_turn_nb = format!(
        "Current turn: {}",
        server_data()
            .core_game_data
            .game_manager
            .game_state
            .current_turn_nb
    );
    let total_order_to_play = format!(
        "Total order to play: {}",
        server_data()
            .core_game_data
            .game_manager
            .game_state
            .order_to_play
            .len()
    );

    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "Game Stats" }
                SheetDescription { "Assess the evolution of the game here." }
            }

            div {
                display: "grid",
                flex: "1 1 0%",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-name", "{current_turn_nb}" }
                    Label { html_for: "sheet-demo-name", "{current_round}" }
                    Label { html_for: "sheet-demo-name", "{total_order_to_play}" }
                }
            }

            SheetFooter {
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Cancel" }
                    },
                }
            }
        }

    }
}

#[component]
fn MenuSheet(s: SheetSide, open_wnd: Signal<bool>, is_saved: Signal<bool>) -> Element {
    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "Menu" }
                SheetDescription { "Modify the parameters or save your game here." }
            }

            div {
                display: "grid",
                flex: "1 1 0%",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-name",
                        if is_saved() {
                            "Saved ✅"
                        } else {
                            ""
                        }
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
                            "Cancel"
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
                display: "grid",
                flex: "1 1 0%",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",
                ScrollArea {
                    width: "100%",
                    height: "30em",
                    border: "1px solid var(--primary-color-6)",
                    border_radius: "0.5em",
                    padding: "0 1em 1em 1em",
                    direction: ScrollDirection::Vertical,
                    tabindex: "0",
                    div { class: "scroll-content",
                        for log in server_data().core_game_data.game_manager.logs.iter() {
                            Label { color: "{log.color}", html_for: "sheet-log", "{log.message}" }
                        }
                    }
                }

            }

            SheetFooter {
                Button { "Save changes" }
                SheetClose {
                    r#as: |attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes, "Cancel" }
                    },
                }
            }
        }

    }
}

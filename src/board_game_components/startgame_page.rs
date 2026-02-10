use crate::common::{Route, SERVER_NAME};
use crate::components::label::Label;
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::websocket_handler::game_state::ServerData;
use crate::{
    application::{self, Application},
    board_game_components::gameboard::GameBoard,
    common::APP,
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        separator::Separator,
        sheet::*,
    },
};
use dioxus::fullstack::{CborEncoding, UseWebsocket};
use dioxus::logger::tracing;
use dioxus::prelude::*;
use lib_rpg::game_state::GameStatus;

/// New game
#[component]
pub fn StartGamePage() -> Element {
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();
    // navigator
    let navigator = use_navigator();

    rsx! {
        if server_data().app.game_manager.game_state.status == GameStatus::EndOfGame {
            h1 { "Game Over" }
            h2 { "Remaining players: {server_data().players_info.len()}" }

            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| {
                    let l_server_data = server_data.read().clone();
                    async move {
                        if !l_server_data.players_info.is_empty() {
                            let _ = socket
                                .send( // for now, we just disconnect the user from the server, but we can implement a better way to handle this in the future
                                    ClientEvent::DisconnectFromServerData(
                                        SERVER_NAME(),
                                        local_login_name_session(),
                                    ),
                                )
                                .await;
                        }
                        navigator.push(Route::Home {});
                    }
                },
                "Quit"
            }
            // TODO if nobody quits -> store the number of players at start? and compare with the number of remaining players to know if we are in a "nobody quits" end of game scenario, and handle it differently (ex: show "nobody quits" message and "replay game" button)
            if server_data().owner_player_name == local_login_name_session() {
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| async move {
                        let _ = socket.send(ClientEvent::ReplayGame(SERVER_NAME())).await;
                    },
                    "Replay game"
                }
            }
        }
        Separator {
            style: "margin: 10px 0; width: 50%;",
            horizontal: true,
            decorative: true,
        }
        div {
            div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",
                Sheets {}
                h4 { "Turn: {server_data().app.game_manager.game_state.current_turn_nb}" }
            }
            Separator {
                style: "margin: 10px 0; width: 50%;",
                horizontal: true,
                decorative: true,
            }
            GameBoard {}
        }
    }
}

#[component]
fn SaveButton(is_saved: Signal<bool>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Destructive,
            onclick: move |_| {
                let gm = APP.read().game_manager.clone();
                async move {
                    tracing::info!("Saving game state...");
                    let path = format!(
                        "{}",
                        &APP
                            .read()
                            .game_manager
                            .game_paths
                            .current_game_dir
                            .join("game_manager.json")
                            .to_string_lossy(),
                    );
                    match application::create_dir(
                            APP.read().game_manager.game_paths.current_game_dir.clone(),
                        )
                        .await
                    {
                        Ok(()) => {
                            tracing::info!("Directory created or already existing successfully")
                        }
                        Err(e) => tracing::info!("Failed to create directory: {}", e),
                    }
                    match application::save(
                            path.to_owned(),
                            serde_json::to_string_pretty(&gm).unwrap(),
                        )
                        .await
                    {
                        Ok(()) => {
                            tracing::trace!("save");
                            is_saved.set(true);
                        }
                        Err(e) => tracing::trace!("{}", e),
                    }
                }
            },
            "Save"
        }
    }
}

#[component]
pub fn Sheets() -> Element {
    let mut open = use_signal(|| false);
    let mut side = use_signal(|| SheetSide::Right);
    let mut use_gm = use_signal(|| false);

    let open_sheet = move |s: SheetSide| {
        move |_| {
            side.set(s);
            open.set(true);
            use_gm.set(true);
        }
    };

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
    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "Inventory" }
                SheetDescription { "Update your equipment here." }
            }

            div {
                display: "grid",
                flex: "1 1 0%",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-name", "Name" }
                    Input { id: "sheet-demo-name", initial_value: "Dioxus" }
                }
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-username", "Username" }
                    Input { id: "sheet-demo-username", initial_value: "@dioxus" }
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

#[component]
fn GameStatsSheet(s: SheetSide) -> Element {
    let gm = &APP.read().game_manager;
    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "Game Stats {gm.game_state.current_round}" }
                SheetDescription { "Assess the evolution of the game here." }
            }

            div {
                display: "grid",
                flex: "1 1 0%",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-name", "Name" }
                    Input { id: "sheet-demo-name", initial_value: "Dioxus" }
                }
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-username", "Username" }
                    Input { id: "sheet-demo-username", initial_value: "@dioxus" }
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
fn MenuSheet(s: SheetSide, open_wnd: Signal<bool>) -> Element {
    let mut is_saved: Signal<bool> = use_signal(|| false);

    if !open_wnd() {
        is_saved.set(false);
    }

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
    rsx! {
        SheetContent { side: s,
            SheetHeader {
                SheetTitle { "Sheet Title" }
                SheetDescription { "Watch the last logs here." }
            }

            div {
                display: "grid",
                flex: "1 1 0%",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-name", "Name" }
                    Input { id: "sheet-demo-name", initial_value: "Dioxus" }
                }
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-username", "Username" }
                    Input { id: "sheet-demo-username", initial_value: "@dioxus" }
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

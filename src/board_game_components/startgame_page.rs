use crate::{
    application,
    board_game_components::gameboard::GameBoard,
    common::{ButtonStatus, APP},
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        sheet::*,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::{label::Label, separator::Separator};

/// New game
#[component]
pub fn StartGamePage() -> Element {
    let mut state = use_signal(|| ButtonStatus::StartGame);
    let mut ready_to_start = use_signal(|| true);
    use_effect(move || {
        if state() == ButtonStatus::StartGame {
            ready_to_start.set(true);
        }
    });

    rsx! {
        if state() == ButtonStatus::ReplayGame {
            Button {
                variant: ButtonVariant::Primary,
                onclick: move |_| async move {
                    ready_to_start.set(false);
                    match application::try_new().await {
                        Ok(app) => {
                            *APP.write() = app;
                            APP.write().game_manager.start_new_game();
                            let _ = APP.write().game_manager.start_new_turn();
                        }
                        Err(_) => println!("no app"),
                    }
                    state.set(ButtonStatus::StartGame);
                },
                "Replay game"
            }
        }
        if state() == ButtonStatus::StartGame && ready_to_start() {
            Separator {
                style: "margin: 10px 0; width: 50%;",
                horizontal: true,
                decorative: true,
            }
            div {
                div { style: "display: flex; flex-direction: row; height: 40px; gap: 10px;",
                    SaveButton {}
                    Sheets {}
                    h4 { "Turn: {APP.write().game_manager.game_state.current_turn_nb}" }
                }
                Separator {
                    style: "margin: 10px 0; width: 50%;",
                    horizontal: true,
                    decorative: true,
                }
                GameBoard { game_status: state }
            }
        } else if state() == ButtonStatus::StartGame && !ready_to_start() {
            h4 { "Loading..." }
        }
    }
}

#[component]
fn SaveButton() -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Destructive,
            onclick: move |_| {
                let gm = APP.read().game_manager.clone();
                async move {
                    println!("Saving game state...");
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
                        Ok(()) => println!("Directory created successfully"),
                        Err(e) => println!("Failed to create directory: {}", e),
                    }
                    match application::save(
                            path.to_owned(),
                            serde_json::to_string_pretty(&gm).unwrap(),
                        )
                        .await
                    {
                        Ok(()) => println!("save"),
                        Err(e) => println!("{}", e),
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

    let open_sheet = move |s: SheetSide| {
        move |_| {
            side.set(s);
            open.set(true);
        }
    };

    rsx! {
        div { display: "flex", gap: "0.5rem",
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetSide::Top),
                "Top"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetSide::Right),
                "Right"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetSide::Bottom),
                "Bottom"
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: open_sheet(SheetSide::Left),
                "Left"
            }
        }
        Sheet { open: open(), on_open_change: move |v| open.set(v),
            SheetContent { side: side(),
                SheetHeader {
                    SheetTitle { "Sheet Title" }
                    SheetDescription { "Sheet description goes here." }
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
                        Input {
                            id: "sheet-demo-username",
                            initial_value: "@dioxus",
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
}

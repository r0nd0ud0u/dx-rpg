use crate::{
    application,
    board_game_components::gameboard::GameBoard,
    common::{ButtonStatus, APP},
    components::button::{Button, ButtonVariant},
};
use dioxus::prelude::*;

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
            div {
                div {
                style: "display: flex; flex-direction: row; height: 40px;",
                h4 { "Turn: {APP.write().game_manager.game_state.current_turn_nb}" }
                SaveButton {}
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

use crate::{
    application,
    common::{ButtonStatus, APP},
    components::gameboard::GameBoard,
};
use dioxus::prelude::*;

/// New game
#[component]
pub fn StartGamePage() -> Element {
    let mut state = use_signal(|| ButtonStatus::StartGame);
    let mut ready_to_start = use_signal(|| true);
    let _ = use_resource(move || async move {
        if state() == ButtonStatus::StartGame {
            ready_to_start.set(true);
        }
    });

    rsx! {
        h4 { "Turn: {APP.read().game_manager.game_state.current_turn_nb}" }
        if state() == ButtonStatus::ReplayGame {
            button {
                onclick: move |_| async move {
                    state.set(ButtonStatus::StartGame);
                    ready_to_start.set(false);
                },
                "Replay game"
            }
        } else if state() == ButtonStatus::StartGame && ready_to_start() {
            SaveButton {}
            GameBoard { game_status: state }
        } else if state() == ButtonStatus::StartGame && !ready_to_start() {
            h4 { "Loading..." }
        }
    }
}

#[component]
fn SaveButton() -> Element {
    rsx! {
        button {
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

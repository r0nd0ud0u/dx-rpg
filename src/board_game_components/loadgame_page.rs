use dioxus::logger::tracing;
use dioxus::prelude::*;
use dioxus_primitives::scroll_area::{ScrollArea, ScrollDirection};

use crate::{
    application,
    common::{Route, APP},
    components::button::{Button, ButtonVariant},
};

#[component]
pub fn LoadGame() -> Element {
    let mut active_button: Signal<i64> = use_signal(|| -1);
    let navigator = use_navigator();

    // get_game_list
    let games_list = use_resource(move || async move {
        // read signal active_button to rerun the resource when active_button is set again.
        println!("active button: {active_button}");
        match application::try_new().await {
            Ok(app) => *APP.write() = app,
            Err(_) => println!("no app"),
        }
        let path_dir = APP.write().game_manager.game_paths.clone();
        match application::get_game_list(path_dir.games_dir).await {
            Ok(games) => {
                for game in &games {
                    println!("Game: {}", game.to_string_lossy());
                }
                games
            }
            Err(e) => {
                println!("Error fetching game list: {}", e);
                vec![]
            }
        }
    });
    let button1_game_list = games_list().clone().unwrap_or_default();
    let button2_game_list = games_list().clone().unwrap_or_default();

    rsx! {
        div { class: "home-container",
            h4 { "Load game" }
            ScrollArea {
                width: "25em",
                height: "10em",
                border: "1px solid var(--primary-color-6)",
                border_radius: "0.5em",
                padding: "0 1em 1em 1em",
                direction: ScrollDirection::Vertical,
                tabindex: "0",
                div { class: "scroll-content",
                    for (index , i) in button1_game_list.clone().iter().enumerate() {
                        Button {
                            variant: if active_button() as usize == index { ButtonVariant::Destructive } else { ButtonVariant::Primary },
                            disabled: active_button() == index as i64,
                            onclick: move |_| async move { active_button.set(index as i64) },
                            "{i.clone().to_string_lossy()}"
                        }
                    }
                }
            }

            Button {
                variant: ButtonVariant::Secondary,
                disabled: active_button() == -1,
                onclick: move |_| {
                    let cur_game = button1_game_list
                        .clone()
                        .get(active_button() as usize)
                        .unwrap()
                        .to_owned();
                    async move {
                        tracing::info!("loading game: {}", cur_game.clone().to_string_lossy());
                        let gm = match application::get_gamemanager_by_game_dir(cur_game.clone())
                            .await
                        {
                            Ok(gm) => gm,
                            Err(e) => {
                                tracing::info!("Error fetching game manager: {}", e);
                                return;
                            }
                        };
                        APP.write().game_manager = gm;
                        navigator.push(Route::StartGamePage {});
                    }
                },
                "Start Game"
            }
            Button {
                variant: ButtonVariant::Secondary,
                disabled: active_button() == -1,
                onclick: move |_| {
                    let cur_game = button2_game_list
                        .clone()
                        .get(active_button() as usize)
                        .unwrap()
                        .to_owned();
                    async move {
                        let _ = match application::delete_game(cur_game.clone()).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::debug!("Error deleting game: {}", e);
                                return;
                            }
                        };
                        active_button.set(-1);
                    }
                },
                "Delete Game"
            }
        }
    }
}

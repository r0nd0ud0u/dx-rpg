use dioxus::prelude::*;

use crate::{
    application,
    common::{Route, APP},
};

#[component]
pub fn LoadGame() -> Element {
    let mut active_button: Signal<i64> = use_signal(|| -1);
    let navigator = use_navigator();
    // get_game_list
    let games_list = use_resource(move || async move {
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
    let games_list = games_list().unwrap_or_default();

    rsx! {
        div { class: "home-container",
            h4 { "Load game" }
            for (index , i) in games_list.iter().enumerate() {
                button {
                    class: "button-lobby-list",
                    disabled: active_button() == index as i64,
                    onclick: move |_| async move { active_button.set(index as i64) },
                    "{i.clone().to_string_lossy()}"
                }
            }
            button {
                onclick: move |_| {
                    let cur_game = games_list.get(active_button() as usize).unwrap().to_owned();
                    async move {
                        let _ = application::log_debug(
                                format!("loading game: {}", cur_game.clone().to_string_lossy()),
                            )
                            .await;
                        let gm = match application::get_gamemanager_by_game_dir(cur_game.clone())
                            .await
                        {
                            Ok(gm) => gm,
                            Err(e) => {
                                println!("Error fetching game manager: {}", e);
                                return;
                            }
                        };
                        APP.write().game_manager = gm;
                        navigator.push(Route::LobbyPage {});
                    }
                },
                "Start Game"
            }
        }
    }
}

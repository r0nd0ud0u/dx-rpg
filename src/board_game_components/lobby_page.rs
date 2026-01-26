use dioxus::prelude::*;

use crate::{
    application,
    board_game_components::common_comp::ButtonLink,
    common::{APP, Route},
};

#[component]
pub fn LobbyPage() -> Element {
    let mut ready_to_start = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            match application::try_new().await {
                Ok(app) => {
                    // init application and game manager
                    *APP.write() = app;
                    // start a new game
                    APP.write().game_manager.start_new_game();
                    let _ = APP.write().game_manager.start_new_turn();
                    // update ongoing games status
                    let all_games_dir = format!(
                        "{}/ongoing-games.json",
                        APP.read()
                            .game_manager
                            .game_paths
                            .games_dir
                            .to_string_lossy()
                    );
                    let mut ongoing_games =
                        match application::read_ongoinggames_from_json(all_games_dir.clone()).await
                        {
                            Ok(og) => og,
                            Err(e) => {
                                println!("Error reading ongoing games: {}", e);
                                application::OngoingGames::default()
                            }
                        };
                    // at the moment only one game is ongoing
                    ongoing_games.all_games.clear();
                    // add the current game directory to ongoing games
                    ongoing_games
                        .all_games
                        .push(APP.read().game_manager.game_paths.current_game_dir.clone());
                    match application::save(
                        all_games_dir,
                        serde_json::to_string_pretty(&ongoing_games).unwrap(),
                    )
                    .await
                    {
                        Ok(_) => println!("Game state saved successfully"),
                        Err(e) => println!("Failed to save game state: {}", e),
                    }
                    // save the game manager state
                    let path = format!(
                        "{}",
                        &APP.read()
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
                        serde_json::to_string_pretty(&APP.read().game_manager.clone()).unwrap(),
                    )
                    .await
                    {
                        Ok(()) => println!("save"),
                        Err(e) => println!("{}", e),
                    }
                    ready_to_start.set(true);
                }
                Err(_) => println!("no app"),
            }
        });
    });

    rsx! {
        div { class: "home-container",
            h1 { "LobbyPage" }
            if ready_to_start() {
                ButtonLink {
                    target: Route::StartGamePage {}.into(),
                    name: "Start Game".to_string(),
                }
            }
        }
    }
}

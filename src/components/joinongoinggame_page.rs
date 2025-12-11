use async_std::task::sleep;
use dioxus::prelude::*;

use crate::{
    application,
    common::{tempo_const::TIMER_FUTURE_1S, Route, APP},
};

#[component]
pub fn JoinOngoingGame() -> Element {
    let _ = use_resource(move || async move {
        match application::try_new().await {
            Ok(app) => {
                *APP.write() = app;
                APP.write().game_manager.start_new_game();
                let _ = APP.write().game_manager.start_new_turn();
            }
            Err(_) => println!("no app"),
        }
    });
    let mut ongoing_games_sig = use_signal(application::OngoingGames::default);
    use_future(move || {
        async move {
            loop {
                // always sleep at start of loop
                sleep(std::time::Duration::from_millis(TIMER_FUTURE_1S)).await;
                // update ongoing games status
                let all_games_dir = format!(
                    "{}/ongoing-games.json",
                    APP.write()
                        .game_manager
                        .game_paths
                        .games_dir
                        .to_string_lossy()
                );
                let ongoing_games =
                    match application::read_ongoinggames_from_json(all_games_dir.clone()).await {
                        Ok(og) => {
                            application::log_debug(format!("ongoing games: {:?}", og.all_games))
                                .await
                                .unwrap();
                            if !og.all_games.is_empty() {
                                match application::get_gamemanager_by_game_dir(
                                    og.all_games[0].clone(),
                                )
                                .await
                                {
                                    Ok(gm) => APP.write().game_manager = gm,
                                    Err(e) => {
                                        application::log_debug(format!(
                                            "Error fetching game manager: {}",
                                            e
                                        ))
                                        .await
                                        .unwrap();
                                    }
                                }
                                og
                            } else {
                                application::log_debug("No ongoing games found".to_string())
                                    .await
                                    .unwrap();
                                application::OngoingGames::default()
                            }
                        }
                        Err(e) => {
                            application::log_debug(format!("Error reading ongoing games: {}", e))
                                .await
                                .unwrap();
                            application::OngoingGames::default()
                        }
                    };
                ongoing_games_sig.set(ongoing_games.clone());
            }
        }
    });
    if !ongoing_games_sig().all_games.is_empty() {
        rsx! {
            div { class: "ongoing-games-container",
                h4 { "Ongoing Games" }
                div { class: "ongoing-game-item",
                    Link { to: Route::StartGamePage {}, "Join Game" }
                    "{ongoing_games_sig().all_games[0].to_string_lossy()}"
                }
            }
        }
    } else {
        rsx! {
            div { "No loading ongoing games" }
        }
    }
}

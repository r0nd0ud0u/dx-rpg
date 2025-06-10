use async_std::task::sleep;
use lib_rpg::{
    attack_type::AttackType, effect::EffectOutcome, game_manager::ResultLaunchAttack,
    game_state::GameStatus,
};

use crate::{
    application::{self, log_debug},
    common::{tempo_const::TIMER_FUTURE_1S, APP},
    components::character_page::{AttackList, CharacterPanel},
};
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonStatus {
    StartGame = 0,
    ReplayGame,
}

#[component]
pub fn GameBoard(game_status: Signal<ButtonStatus>) -> Element {
    let mut current_atk = use_signal(AttackType::default);
    let atk_menu_display = use_signal(|| false);
    let mut write_game_manager = use_signal(|| false);
    let mut reload_app = use_signal(|| false);

    use_future(move || {
        async move {
            loop {
                // always sleep at start of loop
                sleep(std::time::Duration::from_millis(TIMER_FUTURE_1S)).await;
                if APP.write().game_manager.is_round_auto() {
                    sleep(std::time::Duration::from_millis(3000)).await;
                    APP.write().game_manager.game_state.last_result_atk =
                        ResultLaunchAttack::default();
                    APP.write().game_manager.launch_attack("SimpleAtk");
                    log_debug(format!(
                        "launcher  {}",
                        APP.write()
                            .game_manager
                            .game_state
                            .last_result_atk
                            .launcher_name
                    ))
                    .await
                    .unwrap();
                    log_debug(format!(
                        "target  {}",
                        APP.write().game_manager.game_state.last_result_atk.outcomes[0].target_name
                    ))
                    .await
                    .unwrap();
                    current_atk.set(AttackType::default());
                    write_game_manager.set(true);
                }
            }
        }
    });

    // Timer every second to update the game manager by reading json file
    use_future(move || {
        async move {
            loop {
                // always sleep at start of loop
                sleep(std::time::Duration::from_millis(TIMER_FUTURE_1S)).await;
                if !reload_app() {
                    reload_app.set(true);
                }
                if write_game_manager() {
                    write_game_manager.set(false);
                    // save the game manager state
                    let path = format!(
                        "{}",
                        &APP.write()
                            .game_manager
                            .game_paths
                            .current_game_dir
                            .join("game_manager.json")
                            .to_string_lossy(),
                    );
                    let new_dir = APP.write().game_manager.game_paths.current_game_dir.clone();
                    match application::create_dir(new_dir).await {
                        Ok(()) => println!("Directory created successfully"),
                        Err(e) => println!("Failed to create directory: {}", e),
                    }
                    let gm = APP.write().game_manager.clone();
                    match application::save(
                        path.to_owned(),
                        serde_json::to_string_pretty(&gm).unwrap(),
                    )
                    .await
                    {
                        Ok(()) => println!("save"),
                        Err(e) => println!("{}", e),
                    }
                } else if reload_app() {
                    // write the game manager to the app
                    reload_app.set(false);
                    let cur_game_dir = APP.write().game_manager.game_paths.current_game_dir.clone();
                    match application::get_gamemanager_by_game_dir(cur_game_dir.clone()).await {
                        Ok(gm) => APP.write().game_manager = gm,
                        Err(e) => {
                            println!("Error fetching game manager: {}", e)
                        }
                    }
                }
            }
        }
    });

    // Check if the game is at the end of the game and set the game status to ReplayGame
    use_effect(move || {
        if APP.read().game_manager.game_state.status == GameStatus::EndOfGame {
            game_status.set(ButtonStatus::ReplayGame);
        }
    });

    // Display the game board with characters and attacks
    rsx! {
        div { class: "grid-board",
            div {
                // Heroes
                for c in APP.read().game_manager.pm.active_heroes.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_name: APP.read().game_manager.pm.current_player.name.clone(),
                        selected_atk: current_atk,
                        atk_menu_display,
                        write_game_manager,
                        is_auto_atk: false,
                    }
                }
            }
            div {
                "{APP.read().game_manager.game_state.current_turn_nb} "
                if atk_menu_display() {
                    AttackList {
                        name: APP.read().game_manager.pm.current_player.name.clone(),
                        display_atklist_sig: atk_menu_display,
                        selected_atk: current_atk,
                        write_game_manager,
                    }
                } else if !current_atk().name.is_empty() {
                    button {
                        onclick: move |_| async move {
                            APP.write().game_manager.launch_attack(current_atk().name.as_str());
                            current_atk.set(AttackType::default());
                            write_game_manager.set(true);
                        },
                        "launch atk"
                    }
                } else {
                    div {
                        ResultAtkText { ra: APP.read().game_manager.game_state.last_result_atk.clone() }
                    }
                }
            }
            div {
                // Bosses
                for c in APP.read().game_manager.pm.active_bosses.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_name: "",
                        selected_atk: current_atk,
                        atk_menu_display,
                        write_game_manager,
                        is_auto_atk: APP.read().game_manager.pm.current_player.name == c.name,
                    }
                }
            }
        }
    }
}

#[component]
fn ResultAtkText(ra: ResultLaunchAttack) -> Element {
    rsx! {
        "Last round:"
        if !ra.outcomes.is_empty() {
            if ra.is_crit {
                "Critical Strike !"
            }
            for d in ra.all_dodging {
                if d.is_dodging {
                    "{d.name} is dodging"
                } else if d.is_blocking {
                    "{d.name} is blocking"
                }
            }
            for o in ra.outcomes {
                AmountText { eo: o }
            }
        } else {
            "No effects"
        }
    }
}

#[component]
fn AmountText(eo: EffectOutcome) -> Element {
    let mut colortext = "green";
    if eo.real_amount_tx < 0 {
        colortext = "red";
    }
    rsx! {
        div { color: colortext, "{eo.target_name}: {eo.real_amount_tx}" }
    }
}

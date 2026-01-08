use async_std::task::sleep;
use lib_rpg::{
    common::{effect_const::EFFECT_NB_COOL_DOWN, stats_const::HP},
    effect::EffectOutcome,
    game_manager::ResultLaunchAttack,
    game_state::GameStatus,
};

use crate::{
    application::{self, log_debug},
    board_game_components::character_page::{AttackList, CharacterPanel},
    common::{tempo_const::TIMER_FUTURE_1S, ButtonStatus, APP},
    components::button::{Button, ButtonVariant},
};
use dioxus::prelude::*;

#[component]
pub fn GameBoard(game_status: Signal<ButtonStatus>) -> Element {
    let atk_menu_display = use_signal(|| false);
    let mut write_game_manager = use_signal(|| false);
    let mut reload_app = use_signal(|| false);
    let mut selected_atk_name = use_signal(|| "".to_string());

    use_future(move || {
        async move {
            loop {
                // always sleep at start of loop
                sleep(std::time::Duration::from_millis(TIMER_FUTURE_1S)).await;
                // Auto - atk
                if APP.write().game_manager.is_round_auto() {
                    sleep(std::time::Duration::from_millis(3000)).await;
                    APP.write().game_manager.game_state.last_result_atk =
                        ResultLaunchAttack::default();
                    // TODO add other boss attacks
                    // Launch attack
                    APP.write().game_manager.launch_attack("SimpleAtk");
                    log_debug(format!(
                        "launcher  {} {}",
                        APP.write()
                            .game_manager
                            .game_state
                            .last_result_atk
                            .launcher_name,
                        selected_atk_name()
                    ))
                    .await
                    .unwrap();
                    let last_result_atk = &APP.write().game_manager.game_state.last_result_atk;
                    if !last_result_atk.outcomes.is_empty() {
                        log_debug(format!(
                            "target  {}",
                            last_result_atk.outcomes[0].target_name
                        ))
                        .await
                        .unwrap();
                    }
                    // reset atk
                    selected_atk_name.set("".to_string());
                    // update game manager
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
                            application::log_debug(format!("Error fetching game manager: {}", e))
                                .await
                                .unwrap()
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
                        selected_atk_name,
                        atk_menu_display,
                        write_game_manager,
                        is_auto_atk: false,
                    }
                }
            }
            div {
                if atk_menu_display() {
                    AttackList {
                        name: APP.read().game_manager.pm.current_player.name.clone(),
                        display_atklist_sig: atk_menu_display,
                        write_game_manager,
                        selected_atk_name,
                    }
                } else if !selected_atk_name().is_empty() {
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| async move {
                            // launch attack
                            let _ = APP.write().game_manager.launch_attack(&selected_atk_name());
                            log_debug(
                                    format!(
                                        "launcher  {} {}",
                                        APP.write().game_manager.game_state.last_result_atk.launcher_name,
                                        selected_atk_name(),
                                    ),
                                )
                                .await
                                .unwrap();
                            // reset atk
                            selected_atk_name.set("".to_string());
                            // update game manager
                            write_game_manager.set(true);
                        },
                        "launch atk"
                    }
                } else {
                    div {
                        ResultAtkText { ra: APP.read().game_manager.game_state.last_result_atk.clone() }
                    }
                    div {
                        if !APP.read().game_manager.game_state.last_result_atk.logs_new_round.is_empty() {
                            "Starting round:\n"
                            for log in APP.read().game_manager.game_state.last_result_atk.logs_new_round.iter() {
                                "{log}\n"
                            }
                        }
                    }
                }
            }
            div {
                // Bosses
                for c in APP.read().game_manager.pm.active_bosses.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_name: APP.read().game_manager.pm.current_player.name.clone(),
                        selected_atk_name,
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
        if !ra.outcomes.is_empty() {
            "Last attack:\n"
            if ra.is_crit {
                "Critical Strike !"
            }
            for d in ra.all_dodging {
                if d.is_dodging {
                    "{d.name} is dodging\n"
                } else if d.is_blocking {
                    "{d.name} is blocking\n"
                }
            }
            for o in ra.outcomes {
                AmountText { eo: o }
            }
        } else {
            ""
        }
    }
}

#[component]
fn AmountText(eo: EffectOutcome) -> Element {
    let mut colortext = "green";
    if eo.new_effect_param.stats_name == HP && eo.real_hp_amount_tx < 0 {
        colortext = "red";
    } else if eo.full_atk_amount_tx < 0 {
        colortext = "red";
    }
    rsx! {
        if eo.new_effect_param.effect_type == EFFECT_NB_COOL_DOWN {
            div { color: colortext, "{eo.new_effect_param.effect_type}: {eo.new_effect_param.nb_turns}" }
        } else if eo.new_effect_param.stats_name == HP {
            div { color: colortext,
                "{eo.new_effect_param.effect_type}-{eo.new_effect_param.stats_name} {eo.target_name}: {eo.real_hp_amount_tx}"
            }
        } else {
            div { color: colortext,
                "{eo.new_effect_param.effect_type}-{eo.new_effect_param.stats_name} {eo.target_name}: {eo.full_atk_amount_tx}"
            }
        }
    }
}

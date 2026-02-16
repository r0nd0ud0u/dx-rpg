use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
};
use lib_rpg::{
    common::{effect_const::EFFECT_NB_COOL_DOWN, stats_const::HP},
    effect::EffectOutcome,
    game_manager::ResultLaunchAttack,
};

use crate::{
    board_game_components::character_page::{AttackList, CharacterPanel},
    common::SERVER_NAME,
    components::button::{Button, ButtonVariant},
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        game_state::ServerData,
    },
};
use dioxus::prelude::*;

#[component]
pub fn GameBoard() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();

    // local signals
    let atk_menu_display = use_signal(|| false);
    let mut selected_atk_name = use_signal(|| "".to_string());

    // Display the game board with characters and attacks
    rsx! {
        div { class: "grid-board",
            div {
                // Heroes
                for c in server_data.read().app.game_manager.pm.active_heroes.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_name: server_data.read().app.game_manager.pm.current_player.name.clone(),
                        selected_atk_name,
                        atk_menu_display,
                        is_auto_atk: false,
                    }
                }
            }
            div {
                if atk_menu_display() {
                    AttackList {
                        name: server_data.read().app.game_manager.pm.current_player.name.clone(),
                        display_atklist_sig: atk_menu_display,
                        selected_atk_name,
                    }
                } else if !selected_atk_name().is_empty() {
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| async move {
                            tracing::debug!(
                                // reset atk
                                "launcher  {} {}", server_data.read().app.game_manager.game_state
                                .last_result_atk.launcher_name, selected_atk_name()
                            );
                            let _ = socket
                                .send(ClientEvent::LaunchAttack(SERVER_NAME(), selected_atk_name()))
                                .await;
                            selected_atk_name.set("".to_string());
                        },
                        "launch atk"
                    }
                } else {
                    div {
                        ResultAtkText { ra: server_data.read().app.game_manager.game_state.last_result_atk.clone() }
                    }
                    div {
                        if !server_data
                            .read()
                            .app
                            .game_manager
                            .game_state
                            .last_result_atk
                            .logs_new_round
                            .is_empty()
                        {
                            "Starting round:\n"
                            for log in server_data.read().app.game_manager.game_state.last_result_atk.logs_new_round.iter() {
                                "{log}\n"
                            }
                        }
                    }
                }
            }
            div {
                // Bosses
                for c in server_data.read().app.game_manager.pm.active_bosses.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_name: server_data.read().app.game_manager.pm.current_player.name.clone(),
                        selected_atk_name,
                        atk_menu_display,
                        is_auto_atk: server_data.read().app.game_manager.pm.current_player.name == c.name,
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
    if eo.new_effect_param.stats_name == HP && eo.real_hp_amount_tx < 0 || eo.full_atk_amount_tx < 0
    {
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

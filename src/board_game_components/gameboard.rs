use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
};
use lib_rpg::{
    character_mod::buffers::BufKinds,
    common::constants::stats_const::HP,
    server::{
        game_manager::ResultLaunchAttack, players_manager::GameAtkEffect,
        server_manager::ServerData,
    },
};

use crate::{
    board_game_components::character_page::{AttackList, CharacterPanel, PotionList},
    common::SERVER_NAME,
    components::button::{Button, ButtonVariant},
    websocket_handler::event::{ClientEvent, ServerEvent},
};
use dioxus::prelude::*;

#[component]
pub fn GameBoard() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let toggle_atk_animation = use_context::<Signal<bool>>();

    // eval server_data
    if server_data() == ServerData::default() {
        return rsx! {};
    }

    let output_text_css_class = if toggle_atk_animation() {
        ""
    } else {
        "blink-1"
    };

    // local signals
    let atk_menu_display = use_signal(|| false);
    let potion_menu_display = use_signal(|| false);
    let mut selected_atk_name = use_signal(|| "".to_string());

    // spectator: player has no character in active heroes
    let local_session_player_name = use_context::<Signal<String>>();
    let my_character = server_data()
        .players_data
        .get_first_character_name(&local_session_player_name());
    let is_spectator = my_character.as_ref().map_or(true, |char_name| {
        !server_data()
            .core_game_data
            .game_manager
            .pm
            .active_heroes
            .iter()
            .any(|h| &h.id_name == char_name)
    });

    // Display the game board with characters and attacks
    rsx! {
        if is_spectator {
            div { class: "spectator-banner",
                "👁 Spectator mode — you have no active character in this game"
            }
        }
        div { class: "grid-board",
            div {
                // Heroes
                for c in server_data.read().core_game_data.game_manager.pm.active_heroes.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_id_name: server_data.read().core_game_data.game_manager.pm.current_player.id_name.clone(),
                        selected_atk_name,
                        atk_menu_display,
                        potion_menu_display,
                        is_auto_atk: false,
                    }
                }
            }
            div { class: "combat-log",
                if !is_spectator {
                    if atk_menu_display() {
                        AttackList {
                            id_name: server_data.read().core_game_data.game_manager.pm.current_player.id_name.clone(),
                            display_atklist_sig: atk_menu_display,
                            selected_atk_name,
                        }
                    } else if potion_menu_display() {
                        PotionList {
                            id_name: my_character.clone().unwrap_or_default(),
                            display_potionlist_sig: potion_menu_display,
                        }
                    } else if !selected_atk_name().is_empty() {
                        Button {
                            variant: ButtonVariant::Destructive,
                            onclick: move |_| async move {
                                tracing::debug!(
                                    "launcher {} {}", server_data.read().core_game_data.game_manager.game_state
                                    .last_result_atk.launcher_id_name, selected_atk_name()
                                );
                                let _ = socket
                                    .send(ClientEvent::LaunchAttack(SERVER_NAME(), selected_atk_name()))
                                    .await;
                                selected_atk_name.set("".to_string());
                            },
                            "⚔️ Launch Attack"
                        }
                    } else {
                        div { class: "{output_text_css_class}",
                            ResultAtkText { ra: server_data.read().core_game_data.game_manager.game_state.last_result_atk.clone() }
                        }
                        div {
                            if !server_data
                                .read()
                                .core_game_data
                                .game_manager
                                .game_state
                                .last_result_atk
                                .logs_end_of_round
                                .is_empty()
                            {
                                for log in server_data
                                    .read()
                                    .core_game_data
                                    .game_manager
                                    .game_state
                                    .last_result_atk
                                    .logs_end_of_round
                                    .iter()
                                {
                                    "{log.message}\n"
                                }
                            }
                        }
                    }
                } else {
                    div { class: "{output_text_css_class}",
                        ResultAtkText { ra: server_data.read().core_game_data.game_manager.game_state.last_result_atk.clone() }
                    }
                }
            }
            div {
                // Bosses
                for c in server_data.read().core_game_data.game_manager.pm.active_bosses.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_id_name: server_data.read().core_game_data.game_manager.pm.current_player.id_name.clone(),
                        selected_atk_name,
                        atk_menu_display,
                        potion_menu_display,
                        is_auto_atk: server_data.read().core_game_data.game_manager.pm.current_player.id_name
                            == c.id_name,
                    }
                }
            }
        }
    }
}

#[component]
fn ResultAtkText(ra: ResultLaunchAttack) -> Element {
    rsx! {
        if !ra.new_game_atk_effects.is_empty() {
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
            for gae in ra.new_game_atk_effects {
                AmountText { gae: gae.clone() }
            }
        } else {
            ""
        }
    }
}

#[component]
fn AmountText(gae: GameAtkEffect) -> Element {
    let mut colortext = "var(--secondary-success-color)";
    if gae.effect_outcome.real_amount_tx < 0 {
        colortext = "var(--secondary-color-2)";
    }
    rsx! {
        if gae.processed_effect_param.input_effect_param.buffer.kind
            == BufKinds::CooldownTurnsNumber
        {
            div { color: colortext,
                "{gae.processed_effect_param.input_effect_param.buffer.kind}: {gae.processed_effect_param.input_effect_param.buffer.value}"
            }
        } else if gae.processed_effect_param.input_effect_param.buffer.stats_name == HP {
            div { color: colortext,
                "{gae.processed_effect_param.input_effect_param.buffer.kind}-{gae.processed_effect_param.input_effect_param.buffer.stats_name} {gae.effect_outcome.target_id_name}: {gae.effect_outcome.real_amount_tx}"
            }
        } else {
            div { color: colortext,
                "{gae.processed_effect_param.input_effect_param.buffer.kind}-{gae.processed_effect_param.input_effect_param.buffer.stats_name} {gae.effect_outcome.target_id_name}: {gae.effect_outcome.real_amount_tx}"
            }
        }
    }
}

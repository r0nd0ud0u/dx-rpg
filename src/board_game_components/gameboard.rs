use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
};
use lib_rpg::{
    character_mod::buffers::BufKinds,
    server::{
        game_manager::ResultLaunchAttack, players_manager::GameAtkEffect,
        server_manager::ServerData,
    },
};

use crate::{
    board_game_components::character_page::{AttackList, CharacterPanel, PotionList},
    common::{CtxToggleAtkAnimation, SERVER_NAME},
    components::button::{Button, ButtonVariant},
    websocket_handler::event::{ClientEvent, ServerEvent},
};
use dioxus::prelude::*;
use dioxus_i18n::t;

#[component]
pub fn GameBoard() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let toggle_atk_animation = use_context::<CtxToggleAtkAnimation>().0;

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
    let mut selected_atk_name = use_signal(|| "".to_owned());
    // "personal:{name}" or "party:{name}" when waiting for a consumable target, "" otherwise
    let mut selected_consumable = use_signal(|| "".to_owned());
    // id_name of the character currently selected as the consumable target (empty = use server default)
    let mut selected_consumable_target = use_signal(|| "".to_owned());

    // spectator: player has no character in active heroes
    let local_session_player_name = use_context::<Signal<String>>();
    let my_character = server_data()
        .players_data
        .get_first_character_name(&local_session_player_name());
    // In single-player mode, the one real player controls all heroes — never spectator
    let is_spectator = if server_data().core_game_data.is_single_player {
        false
    } else {
        my_character.as_ref().is_none_or(|char_name| {
            !server_data()
                .core_game_data
                .game_manager
                .pm
                .active_heroes
                .iter()
                .any(|h| &h.id_name == char_name)
        })
    };

    // Display the game board with characters and attacks
    rsx! {
        if is_spectator {
            div { class: "spectator-banner", {t!("gameboard-spectator-mode")} }
        }
        div { class: "grid-board",
            div {
                // Heroes
                for c in server_data.read().core_game_data.game_manager.pm.active_heroes.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_id_name: server_data.read().core_game_data.game_manager.pm.current_player.id_name.clone(),
                        selected_atk_name,
                        selected_consumable,
                        selected_consumable_target,
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
                        // The potion menu acts for the hero whose turn it is, so show
                        // that hero's potions (matching AttackList above) rather than the
                        // session's first character.
                        PotionList {
                            id_name: server_data.read().core_game_data.game_manager.pm.current_player.id_name.clone(),
                            display_potionlist_sig: potion_menu_display,
                            selected_consumable,
                        }
                    } else if !selected_consumable().is_empty() {
                        {
                            // Resolve target: prefer the locally-selected one, fall back
                            // to the server's default is_current_target.
                            let snap = server_data.read();
                            let pm = &snap.core_game_data.game_manager.pm;
                            let server_default = pm
                                .active_heroes
                                .iter()
                                .chain(pm.active_bosses.iter())
                                .find(|c| c.character_rounds_info.is_current_target)
                                .map(|c| c.id_name.clone());
                            drop(snap);
                            let resolved_target = if !selected_consumable_target().is_empty() {
                                Some(selected_consumable_target())
                            } else {
                                server_default
                            };
                            let local_player = use_context::<Signal<String>>()();
                            let consumable_val = selected_consumable();
                            rsx! {
                                if let Some(target_id) = resolved_target {
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| {
                                            let tid = target_id.clone();
                                            let cval = consumable_val.clone();
                                            let player = local_player.clone();
                                            async move {
                                                let (is_party, name) = if let Some(n) = cval.strip_prefix("party:") {
                                                    (true, n.to_owned())
                                                } else {
                                                    (false, cval.trim_start_matches("personal:").to_owned())
                                                };
                                                if is_party {
                                                    let _ = socket
                                                        .send(ClientEvent::UsePartyPotion(SERVER_NAME(), player, name, tid))
                                                        .await;
                                                } else {
                                                    let _ = socket
                                                        .send(ClientEvent::UsePotion(SERVER_NAME(), player, name, tid))
                                                        .await;
                                                }
                                                selected_consumable.set("".to_owned());
                                                selected_consumable_target.set("".to_owned());
                                            }
                                        },
                                        {t!("gameboard-use")}
                                    }
                                }
                            }
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
                                selected_atk_name.set("".to_owned());
                            },
                            {t!("gameboard-launch-attack")}
                        }
                    } else {
                        {
                            let snap = server_data.read();
                            let ra = snap.core_game_data.game_manager.game_state.last_result_atk.clone();
                            let is_boss_atk = snap
                                .core_game_data
                                .game_manager
                                .pm
                                .active_bosses
                                .iter()
                                .any(|b| b.id_name == ra.launcher_id_name);
                            let last_action_header = snap.core_game_data.last_action_header.clone();
                            drop(snap);
                            let banner_class = if is_boss_atk {
                                "boss-atk-banner"
                            } else {
                                "hero-atk-banner"
                            };
                            rsx! {
                                if !ra.atk_name.is_empty() {
                                    div { class: "{banner_class} {output_text_css_class}",
                                        {t!("gameboard-attacks", launcher : ra.launcher_id_name.clone())}
                                    }
                                } else if !last_action_header.is_empty() {
                                    div { class: "action-banner {output_text_css_class}", "{last_action_header}" }
                                }
                                if !ra.logs_end_of_round.is_empty() {
                                    div { class: "round-log-header",
                                        {
                                            t!(
                                                "gameboard-turn-round", turn : ra.turn_nb, round : ra
                                                .round_nb
                                            )
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "{output_text_css_class}",
                            ResultAtkText { ra: server_data.read().core_game_data.game_manager.game_state.last_result_atk.clone() }
                        }
                        div {
                            {
                                let logs = server_data
                                    .read()
                                    .core_game_data
                                    .game_manager
                                    .game_state
                                    .last_result_atk
                                    .logs_end_of_round
                                    .clone();
                                if !logs.is_empty() {
                                    rsx! {
                                        for log in logs.iter() {
                                            {
                                                let msg = log.message.replace('\n', "<br/>");
                                                rsx! {
                                                    div {
                                                        style: "color: {log.color}; font-size: 0.82rem; padding: 1px 0;",
                                                        dangerous_inner_html: "{msg}",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {}
                                }
                            }
                        }
                    }
                } else {
                    {
                        let snap = server_data.read();
                        let ra = snap.core_game_data.game_manager.game_state.last_result_atk.clone();
                        let is_boss_atk = snap
                            .core_game_data
                            .game_manager
                            .pm
                            .active_bosses
                            .iter()
                            .any(|b| b.id_name == ra.launcher_id_name);
                        let last_action_header = snap.core_game_data.last_action_header.clone();
                        drop(snap);
                        let banner_class = if is_boss_atk {
                            "boss-atk-banner"
                        } else {
                            "hero-atk-banner"
                        };
                        rsx! {
                            if !ra.atk_name.is_empty() {
                                div { class: "{banner_class} {output_text_css_class}", "⚔️ {ra.launcher_id_name} attacks!" }
                            } else if !last_action_header.is_empty() {
                                div { class: "action-banner {output_text_css_class}", "{last_action_header}" }
                            }
                        }
                    }
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
                        selected_consumable,
                        selected_consumable_target,
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
pub fn ResultAtkText(ra: ResultLaunchAttack) -> Element {
    // `all_dodging` holds one entry per attack effect, so a target that dodges an
    // attack with several effects appears multiple times. Deduplicate by character
    // name so each dodge/block is reported only once.
    let mut seen_dodging = std::collections::HashSet::new();
    let dodging: Vec<_> = ra
        .all_dodging
        .iter()
        .filter(|d| (d.is_dodging || d.is_blocking) && seen_dodging.insert(d.name.clone()))
        .cloned()
        .collect();

    // Show "Last attack" block whenever there are effects OR at least one dodge/block to report.
    let has_dodge_info = !dodging.is_empty();

    // Group effects by target, preserving the order of first appearance.
    let mut ordered_groups: Vec<(String, Vec<GameAtkEffect>)> = Vec::new();
    for gae in &ra.new_game_atk_effects {
        let target = gae.effect_outcome.target_id_name.clone();
        if let Some(group) = ordered_groups.iter_mut().find(|(t, _)| t == &target) {
            group.1.push(gae.clone());
        } else {
            ordered_groups.push((target, vec![gae.clone()]));
        }
    }

    let show_block =
        !ra.new_game_atk_effects.is_empty() || has_dodge_info || !ra.passive_logs.is_empty();
    rsx! {
        if show_block {
            {t!("gameboard-last-attack", name : ra.atk_name.clone())}
            "\n"
            if ra.is_crit {
                div { style: "color: var(--secondary-color-2); font-weight: bold; font-size: 1.1em;",
                    {t!("gameboard-critical-strike")}
                }
            }
            for d in dodging {
                if d.is_dodging {
                    {t!("gameboard-is-dodging", name : d.name.clone())}
                    "\n"
                } else if d.is_blocking {
                    {t!("gameboard-is-blocking", name : d.name.clone())}
                    "\n"
                }
            }
            for (i, (_target, effects)) in ordered_groups.iter().enumerate() {
                if i > 0 {
                    hr { style: "border: none; border-top: 1px dashed var(--border-color); margin: 2px 0;" }
                }
                for gae in effects {
                    AmountText { gae: gae.clone() }
                }
            }
            for log in ra.passive_logs.iter() {
                {
                    let msg = log.message.replace('\n', "<br/>");
                    rsx! {
                        div {
                            style: "color: {log.color}; font-size: 0.85rem; padding: 1px 0;",
                            dangerous_inner_html: "{msg}",
                        }
                    }
                }
            }
        } else {
            ""
        }
    }
}

#[component]
fn AmountText(gae: GameAtkEffect) -> Element {
    let Some(text) = gae.log_text() else {
        return rsx! {};
    };

    let kind = &gae.processed_effect_param.input_effect_param.buffer.kind;
    let mut colortext = "var(--secondary-success-color)";
    if gae.effect_outcome.real_amount_tx < 0 {
        colortext = "var(--secondary-color-2)";
    }
    if *kind == BufKinds::ConditionDamagePrevTurn
        && gae.processed_effect_param.number_of_applies == 0
    {
        colortext = "var(--secondary-color-2)";
    }
    let crit_style = if gae.effect_outcome.is_critical {
        "font-weight: bold; text-decoration: underline;"
    } else {
        ""
    };

    rsx! {
        div { color: colortext, style: crit_style, "{text}" }
    }
}

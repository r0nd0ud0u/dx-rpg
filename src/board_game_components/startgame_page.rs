use crate::board_game_components::character_page::{BarComponent, CharacterPanel};
use crate::board_game_components::game_sheets::{GameSheets, StoreSheet};
use crate::board_game_components::overworld::OverworldMap;
use crate::common::{CtxAutoSaveScenario, Route, SERVER_NAME, photo_src};
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::websocket_handler::msg_from_client::send_disconnect_from_server_data;
use crate::{
    board_game_components::gameboard::GameBoard,
    components::{
        button::{Button, ButtonVariant},
        separator::Separator,
        sheet::{Sheet, SheetSide},
    },
};
use dioxus::fullstack::{CborEncoding, UseWebsocket};
use dioxus::prelude::*;
use lib_rpg::{
    character_mod::character::CharacterKind,
    common::constants::stats_const::HP,
    server::{
        game_state::GameStatus,
        server_manager::{GamePhase, ServerData},
    },
};

/// Read-only character panels shown at end of scenario / game.
#[component]
fn EndStatePanels() -> Element {
    let server_data = use_context::<Signal<ServerData>>();
    let atk_menu = use_signal(|| false);
    let potion_menu = use_signal(|| false);
    let selected_atk = use_signal(|| "".to_owned());
    let selected_consumable = use_signal(|| "".to_owned());
    let selected_consumable_target = use_signal(|| "".to_owned());

    rsx! {
        div { class: "grid-board",
            div {
                for c in server_data().core_game_data.game_manager.pm.active_heroes.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_id_name: "".to_owned(),
                        selected_atk_name: selected_atk,
                        selected_consumable,
                        selected_consumable_target,
                        atk_menu_display: atk_menu,
                        potion_menu_display: potion_menu,
                        is_auto_atk: false,
                    }
                }
            }
            div {}
            div {
                for c in server_data()
                    .core_game_data
                    .game_manager
                    .pm
                    .active_bosses
                    .iter()
                    .filter(|c| c.kind == CharacterKind::Boss)
                {
                    div {
                        class: "character",
                        background_color: "var(--secondary-error-color)",
                        div { class: "char-header",
                            span { class: "char-name-text", "{c.db_full_name}" }
                            span { class: "char-level", "Lvl {c.level}" }
                            if c.stats.is_dead().is_some_and(|v| v) {
                                span { style: "font-size:0.8rem; color:var(--rpg-text-muted);",
                                    "💀 Defeated"
                                }
                            }
                        }
                        div { class: "char-body",
                            img {
                                src: photo_src(&c.photo_name),
                                class: "image-small",
                            }
                            div { class: "character-energy-effects-box",
                                BarComponent {
                                    max: c.stats.all_stats[HP].max,
                                    current: c.stats.all_stats[HP].current,
                                    name: HP.to_owned(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn QuitGameButton() -> Element {
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    rsx! {
        Button {
            variant: ButtonVariant::Destructive,
            onclick: move |_| {
                async move {
                    send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                    let navigator = use_navigator();
                    navigator.push(Route::Home {});
                }
            },
            "🚪 Quit"
        }
    }
}

/// New game
#[component]
pub fn RunningGamePage() -> Element {
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let auto_save_scenario = use_context::<CtxAutoSaveScenario>().0;
    // Shop panel — only open at end-of-scenario
    let mut shop_open = use_signal(|| false);

    let snap_server_data = server_data();

    // Use the server's game_phase as the single source of truth for the overworld view.
    let in_overworld = server_data().core_game_data.game_phase == GamePhase::Overworld;

    // Map the universe to its starting overworld map id.
    let universe_map = |universe: &str| -> Option<&'static str> {
        match universe {
            "lotr" => Some("lotr_shire"),
            "pokemon" => Some("pallet_town"),
            _ => None,
        }
    };

    // Map to return to when the player clicks "Overworld": prefer the current saved map
    // (so a fight triggered on route_1 returns to route_1, not pallet_town), fall back
    // to the universe's starting map when no overworld state exists yet.
    let return_map_id: Option<String> = snap_server_data
        .core_game_data
        .overworld
        .as_ref()
        .map(|ow| ow.map_id.clone())
        .or_else(|| universe_map(&snap_server_data.core_game_data.universe).map(str::to_owned));

    // Auto-enter overworld the first time the game reaches Running phase for
    // universes that have an overworld map, and no saved overworld state exists yet.
    let mut auto_entered = use_signal(|| false);
    use_effect(move || {
        if auto_entered() {
            return;
        }
        let phase = server_data().core_game_data.game_phase.clone();
        let no_overworld = server_data().core_game_data.overworld.is_none();
        let universe = server_data().core_game_data.universe.clone();
        if phase == GamePhase::Running && no_overworld {
            if let Some(map_id) = universe_map(&universe) {
                auto_entered.set(true);
                let server_name = SERVER_NAME();
                let map_id = map_id.to_owned();
                let socket = socket;
                spawn(async move {
                    let _ = socket
                        .send(ClientEvent::EnterOverworld(server_name, map_id))
                        .await;
                });
            }
        }
    });

    rsx! {
        if in_overworld {
            div { class: "game-toolbar",
                GameSheets {}
            }
            OverworldMap {}
        }
        if !in_overworld {
        if server_data().core_game_data.game_manager.game_state.status == GameStatus::EndOfGame {
            div { class: "gameover-page",
                h1 { class: "gameover-title", "💀 Game Over" }
                p { class: "gameover-sub",
                    "Remaining players: {server_data().players_data.players_info.len()}"
                }
                EndStatePanels {}
                div { class: "scenario-actions",
                    QuitGameButton {}
                    if server_data().players_data.owner_player_name == local_login_name_session() {
                        Button {
                            variant: ButtonVariant::GreenType,
                            onclick: move |_| async move {
                                let _ = socket.send(ClientEvent::ReplayGame(SERVER_NAME())).await;
                            },
                            "🔄 Replay Game"
                        }
                    }
                }
            }
        }
        if server_data().core_game_data.game_manager.game_state.status
            == GameStatus::EndOfScenario
        {
            div { class: "scenario-end-page",
                h2 { class: "scenario-end-title", "🏆 Scenario Complete!" }

                // Show the finishing blow details
                {
                    let last_atk = snap_server_data
                        .core_game_data
                        .game_manager
                        .game_state
                        .last_result_atk
                        .clone();
                    let show_blow = !last_atk.new_game_atk_effects.is_empty()
                        || last_atk.is_dot_kill;
                    if show_blow {
                        let title = if last_atk.is_dot_kill {
                            "⚔️ Finishing Blow (DOT)"
                        } else {
                            "⚔️ Finishing Blow"
                        };
                        let dying_last = last_atk.dying_char_last_atk.clone();
                        rsx! {
                            div { class: "scenario-section",
                                h3 { class: "scenario-section-title", "{title}" }
                                if last_atk.is_dot_kill && !dying_last.is_empty() {
                                    p { class: "dot-kill-info", "Enemy's last attack: {dying_last}" }
                                }
                                if !last_atk.new_game_atk_effects.is_empty() {
                                    div { class: "scenario-last-atk",
                                        crate::board_game_components::gameboard::ResultAtkText { ra: last_atk }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }

                EndStatePanels {}
                div { class: "scenario-actions",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| shop_open.set(true),
                        "🛒 Shop"
                    }
                    if server_data().players_data.owner_player_name == local_login_name_session() {
                        Button {
                            variant: ButtonVariant::GreenType,
                            onclick: move |_| async move {
                                let _ = socket
                                    .send(ClientEvent::LoadNextScenario(SERVER_NAME(), auto_save_scenario()))
                                    .await;
                            },
                            "⚡ Load Next Scenario"
                        }
                        if let Some(map_id) = return_map_id.clone() {
                            Button {
                                variant: ButtonVariant::GreenType,
                                onclick: move |_| {
                                    let m = map_id.clone();
                                    async move {
                                        let _ = socket
                                            .send(ClientEvent::EnterOverworld(SERVER_NAME(), m))
                                            .await;
                                    }
                                },
                                "🗺 Explore Overworld"
                            }
                        }
                    }
                    QuitGameButton {}
                }
                Sheet {
                    open: shop_open(),
                    on_open_change: move |v| shop_open.set(v),
                    StoreSheet { s: SheetSide::Right }
                }
                div { class: "scenario-section",
                    h3 { class: "scenario-section-title", "🎁 Loots" }
                    div { class: "loot-grid",
                        for l in snap_server_data.core_game_data.game_manager.current_scenario.loots.iter() {
                            div { class: "loot-item", "{l.format_loot()}" }
                        }
                    }
                }
                div { class: "scenario-section",
                    h3 { class: "scenario-section-title", "⬆️ Level Upgrades" }
                    div {
                        class: "level-up-box",
                        dangerous_inner_html: "{snap_server_data.core_game_data.game_manager.end_of_scenario.to_formatted_string(true)}",
                    }
                }
            }
        } else {
            Separator {
                style: "margin: 10px 0;",
                horizontal: true,
                decorative: true,
            }
            div {
                div { class: "game-toolbar",
                    GameSheets {}
                    div { class: "turn-badge",
                        "⚔️ Turn {server_data().core_game_data.game_manager.game_state.current_turn_nb} - Round {server_data().core_game_data.game_manager.game_state.current_round}"
                    }
                    if server_data().players_data.owner_player_name == local_login_name_session() {
                        if let Some(map_id) = return_map_id.clone() {
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    let m = map_id.clone();
                                    async move {
                                        let _ = socket
                                            .send(ClientEvent::EnterOverworld(SERVER_NAME(), m))
                                            .await;
                                    }
                                },
                                "🗺 Overworld"
                            }
                        }
                    }
                }
                Separator {
                    style: "margin: 10px 0;",
                    horizontal: true,
                    decorative: true,
                }
                GameBoard {}
            }
        }
        } // end if !in_overworld

    }
}

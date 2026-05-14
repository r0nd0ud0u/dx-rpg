use crate::board_game_components::character_page::{BarComponent, CharacterPanel};
use crate::board_game_components::game_sheets::GameSheets;
use crate::common::{Route, SERVER_NAME};
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::websocket_handler::msg_from_client::send_disconnect_from_server_data;
use crate::{
    board_game_components::gameboard::GameBoard,
    components::{
        button::{Button, ButtonVariant},
        separator::Separator,
    },
};
use dioxus::fullstack::{CborEncoding, UseWebsocket};
use dioxus::prelude::*;
use lib_rpg::{
    character_mod::character::CharacterKind,
    common::constants::stats_const::HP,
    server::{game_state::GameStatus, server_manager::ServerData},
};

/// Read-only character panels shown at end of scenario / game.
#[component]
fn EndStatePanels() -> Element {
    let server_data = use_context::<Signal<ServerData>>();
    let atk_menu = use_signal(|| false);
    let potion_menu = use_signal(|| false);
    let selected_atk = use_signal(|| "".to_string());

    rsx! {
        div { class: "grid-board",
            div {
                for c in server_data().core_game_data.game_manager.pm.active_heroes.iter() {
                    CharacterPanel {
                        c: c.clone(),
                        current_player_id_name: "".to_string(),
                        selected_atk_name: selected_atk,
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
                        }
                        div { class: "char-body",
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

    let snap_server_data = server_data();

    rsx! {
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
                EndStatePanels {}
                div { class: "scenario-actions",
                    if server_data().players_data.owner_player_name == local_login_name_session() {
                        Button {
                            variant: ButtonVariant::GreenType,
                            onclick: move |_| async move {
                                let _ = socket.send(ClientEvent::LoadNextScenario(SERVER_NAME())).await;
                            },
                            "⚡ Load Next Scenario"
                        }
                    }
                    QuitGameButton {}
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
                        "⚔️ Turn {server_data().core_game_data.game_manager.game_state.current_turn_nb}"
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

    }
}

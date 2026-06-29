use dioxus::logger::tracing;
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::components::button::ButtonVariant;
use crate::{
    auth_manager::server_fn::list_universes_server,
    board_game_components::{
        character_select::CharacterSelect, common_comp::ButtonLink, startgame_page::RunningGamePage,
    },
    common::{Route, SERVER_NAME},
    components::button::Button,
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::send_start_game,
    },
};

#[component]
pub fn LobbyPage() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let server_data = use_context::<Signal<ServerData>>();

    // Universe selection
    let mut selected_universe = use_signal(String::new);
    let universes_resource = use_resource(list_universes_server);

    // Pre-select the saved universe (if loading an existing game)
    let saved_universe = server_data().core_game_data.universe.clone();
    use_effect(move || {
        let u = saved_universe.clone();
        if !u.is_empty() && selected_universe().is_empty() {
            selected_universe.set(u);
        }
    });
    // A loaded game already has a universe that should not be changed
    let universe_locked = server_data().core_game_data.loaded_from_save;

    // all players info have a character name
    let server_data_snap = server_data();

    let all_players_have_character_name = if server_data_snap.players_data.players_info.is_empty() {
        false
    } else {
        server_data_snap
            .players_data
            .players_info
            .values()
            .all(|p| !p.character_id_names.is_empty())
    };
    tracing::trace!(
        "all_players_have_character_name: {}",
        all_players_have_character_name
    );

    rsx! {
        // if the game is not running, show the lobby page, otherwise show the start game page
        if server_data_snap.core_game_data.game_phase == GamePhase::InitGame
            || server_data_snap.core_game_data.game_phase == GamePhase::Loading
        {
            div { class: "lobby-page",
                h2 { class: "rpg-title",
                    if server_data_snap.core_game_data.game_phase == GamePhase::InitGame {
                        "⚔️ Lobby"
                    } else {
                        "⏳ Loading…"
                    }
                }

                // Info bar: server + players + universe
                div { class: "lobby-info-bar",
                    div { class: "lobby-info-item",
                        span { class: "lobby-info-label", "Server" }
                        span { class: "lobby-info-value", "{SERVER_NAME()}" }
                    }
                    div { class: "lobby-info-item",
                        span { class: "lobby-info-label", "Players" }
                        span { class: "lobby-info-value",
                            "{server_data_snap.players_data.players_info.len()} / {server_data_snap.core_game_data.players_nb}"
                        }
                    }
                    {
                        if !selected_universe().is_empty() {
                            rsx! {
                                div { class: "lobby-info-item",
                                    span { class: "lobby-info-label", "Universe" }
                                    span { class: "lobby-info-value lobby-universe", "🌐 {selected_universe()}" }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                    {
                        let nb = if selected_universe().is_empty() {
                            server_data_snap.core_game_data.game_manager.all_scenarios.len()
                        } else {
                            server_data_snap
                                .core_game_data
                                .game_manager
                                .all_scenarios
                                .iter()
                                .filter(|s| s.universe == selected_universe())
                                .count()
                        };
                        rsx! {
                            div { class: "lobby-info-item",
                                span { class: "lobby-info-label", "Scenarios" }
                                span { class: "lobby-info-value", "{nb}" }
                            }
                        }
                    }
                }

                // Start game button (host only, when all players have picked a character)
                if SERVER_NAME() == local_login_name_session() && all_players_have_character_name
                    && (server_data_snap.core_game_data.game_phase == GamePhase::InitGame
                        || server_data_snap.core_game_data.game_phase == GamePhase::Loading
                            && server_data_snap.core_game_data.players_nb
                                == server_data_snap.players_data.players_info.len() as i64)
                {
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| async move {
                            send_start_game(socket).await;
                        },
                        "▶ Start Game"
                    }
                }

                // Universe selector — hidden when loading a saved game (universe already fixed)
                {
                    let universes = universes_resource
                        .read()
                        .as_ref()
                        .and_then(|r| r.as_ref().ok())
                        .cloned()
                        .unwrap_or_default();
                    if universe_locked {
                        rsx! {
                            div { class: "lobby-universe-select",
                                label { class: "lobby-info-label", "Universe (saved)" }
                                div { class: "lobby-universe-locked", "🔒 {selected_universe()}" }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "lobby-universe-select",
                                label { class: "lobby-info-label", "Choose Universe" }
                                select {
                                    class: "lobby-select",
                                    value: "{selected_universe}",
                                    onchange: move |e| {
                                        let v = e.value();
                                        selected_universe.set(v.clone());
                                        spawn(async move {
                                            let _ = socket
                                                .send(ClientEvent::SetUniverse(SERVER_NAME(), v))
                                                .await;
                                        });
                                    },
                                    option { value: "", "— select a universe —" }
                                    for u in &universes {
                                        option { value: "{u}", "{u}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Character selection — only shown once a universe is chosen
                if !selected_universe().is_empty() {
                    CharacterSelect { universe: selected_universe() }
                }
            }
        } else if server_data_snap.core_game_data.game_phase == GamePhase::Running
            || server_data_snap.core_game_data.game_phase == GamePhase::Overworld
        {
            // check if there is more characters in game than users (skip in single-player mode)
            if server_data_snap.core_game_data.is_single_player
                || server_data_snap.core_game_data.game_manager.pm.active_heroes.len()
                    <= server_data_snap.players_data.players_info.len()
            {
                RunningGamePage {}
            } else {

                ButtonLink {
                    target: Route::Home {}.into(),
                    name: "Not enough players".to_owned(),
                    onclick: move |_| {
                        async move {
                            let _ = socket
                                .send(
                                    ClientEvent::DisconnectFromServerData(
                                        SERVER_NAME(),
                                        local_login_name_session(),
                                    ),
                                )
                                .await;
                        }
                    },
                }
            }
        } else if server_data_snap.core_game_data.game_phase == GamePhase::Ended {
            ButtonLink {
                target: Route::Home {}.into(),
                name: "No more game, back to home".to_owned(),
            }
        } else if server_data_snap.core_game_data.game_phase == GamePhase::Default {

        }
    }
}

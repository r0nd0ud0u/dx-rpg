use dioxus::fullstack::{CborEncoding, UseWebsocket};
use dioxus::html::Key;
use dioxus::prelude::*;
use lib_rpg::{
    common::overworld::{Direction, TileKind},
    server::server_manager::ServerData,
};

use crate::{
    common::SERVER_NAME,
    components::button::{Button, ButtonVariant},
    websocket_handler::event::{ClientEvent, ServerEvent},
};

const TILE_PX: i32 = 48;

fn tile_css(kind: &TileKind) -> &'static str {
    match kind {
        TileKind::Floor => "ow-tile ow-floor",
        TileKind::Wall => "ow-tile ow-wall",
        TileKind::Grass => "ow-tile ow-grass",
        TileKind::Water => "ow-tile ow-water",
        TileKind::Door { .. } => "ow-tile ow-door",
    }
}

fn tile_emoji(kind: &TileKind, locked: bool) -> &'static str {
    match kind {
        TileKind::Floor => "",
        TileKind::Wall => "🧱",
        TileKind::Grass => "",
        TileKind::Water => "💧",
        TileKind::Door { .. } => {
            if locked {
                "🔒"
            } else {
                "🚪"
            }
        }
    }
}

fn is_door(kind: &TileKind) -> bool {
    matches!(kind, TileKind::Door { .. })
}

fn tile_emoji_at(
    kind: &TileKind,
    x: usize,
    y: usize,
    locked_doors: &std::collections::HashSet<String>,
) -> &'static str {
    tile_emoji(
        kind,
        is_door(kind) && locked_doors.contains(&format!("{}_{}", x, y)),
    )
}

fn npc_emoji(npc: &lib_rpg::server::overworld_manager::NpcState) -> &'static str {
    if npc.fight_scenario_id.is_some() {
        "👹"
    } else {
        "🧓"
    }
}

#[component]
pub fn OverworldMap() -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let ow_state = server_data().core_game_data.overworld.clone();
    let Some(ow) = ow_state else {
        return rsx! {
            div {
                class: "ow-container",
                style: "justify-content:center; align-items:center; min-height:300px;",
                p { style: "color:#7fff7f; font-size:1.2rem;", "🗺 Entering overworld…" }
            }
        };
    };

    // Clone socket + context values for the D-pad closures (each needs its own copy).
    let socket_up = socket.clone();
    let socket_down = socket.clone();
    let socket_left = socket.clone();
    let socket_right = socket.clone();
    let socket_interact = socket.clone();
    let socket_confirm_fight = socket.clone();
    let socket_dismiss = socket.clone();

    rsx! {
        div {
            class: "ow-container",
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: move |e: KeyboardEvent| async move {
                let server_name = SERVER_NAME();
                let player_name = local_login_name_session();
                match e.key() {
                    Key::ArrowUp => {
                        e.prevent_default();
                        let _ = socket
                            .send(ClientEvent::MovePlayer(server_name, player_name, Direction::Up))
                            .await;
                    }
                    Key::ArrowDown => {
                        e.prevent_default();
                        let _ = socket
                            .send(ClientEvent::MovePlayer(server_name, player_name, Direction::Down))
                            .await;
                    }
                    Key::ArrowLeft => {
                        e.prevent_default();
                        let _ = socket
                            .send(ClientEvent::MovePlayer(server_name, player_name, Direction::Left))
                            .await;
                    }
                    Key::ArrowRight => {
                        e.prevent_default();
                        let _ = socket
                            .send(ClientEvent::MovePlayer(server_name, player_name, Direction::Right))
                            .await;
                    }
                    Key::Enter => {
                        let _ = socket
                            .send(ClientEvent::Interact(server_name, player_name))
                            .await;
                    }
                    Key::Character(s) if s == " " => {
                        let _ = socket
                            .send(ClientEvent::Interact(server_name, player_name))
                            .await;
                    }
                    _ => {}
                }
            },

            // Scrollable wrapper so the map stays at full resolution on small screens.
            div { class: "ow-grid-scroll",
                div {
                    class: "ow-grid",
                    style: "width: {ow.width * TILE_PX}px;",

                    for (y, row) in ow.tiles.iter().enumerate() {
                        for (x, tile_kind) in row.iter().enumerate() {
                            div {
                                class: "{tile_css(tile_kind)}",
                                style: "display:flex; align-items:center; justify-content:center; font-size:1.4rem; user-select:none;",
                                "{tile_emoji_at(tile_kind, x, y, &ow.locked_doors)}"
                            }
                        }
                    }

                    // Defeated boss NPCs are hidden.
                    for npc in ow.npcs.iter() {
                        if !npc.defeated {
                            div {
                                class: "ow-sprite ow-npc",
                                style: "left:{npc.pos.x * TILE_PX}px; top:{npc.pos.y * TILE_PX}px; width:{TILE_PX}px; height:{TILE_PX}px;",
                                "{npc_emoji(npc)}"
                            }
                        }
                    }

                    for (hero_id , pos) in ow.player_positions.iter() {
                        div {
                            key: "{hero_id}",
                            class: "ow-sprite ow-hero",
                            style: "left:{pos.x * TILE_PX}px; top:{pos.y * TILE_PX}px; width:{TILE_PX}px; height:{TILE_PX}px;",
                            "🧑"
                        }
                    }
                }
            }

            if !ow.active_dialog.is_empty() {
                div { class: "ow-dialog",
                    for line in ow.active_dialog.iter() {
                        p { class: "ow-dialog-line", "{line}" }
                    }
                    if ow.pending_fight.is_some() {
                        p { class: "ow-dialog-question", "Do you want to start the fight?" }
                        div { class: "ow-dialog-actions",
                            button {
                                class: "ow-dialog-btn ow-dialog-btn-yes",
                                onclick: move |_| {
                                    let sn = SERVER_NAME();
                                    let pn = local_login_name_session();
                                    let sock = socket_confirm_fight.clone();
                                    async move {
                                        let _ = sock.send(ClientEvent::Interact(sn, pn)).await;
                                    }
                                },
                                "⚔️ Yes, fight!"
                            }
                            button {
                                class: "ow-dialog-btn ow-dialog-btn-no",
                                onclick: move |_| {
                                    let sn = SERVER_NAME();
                                    let pn = local_login_name_session();
                                    let sock = socket_dismiss.clone();
                                    async move {
                                        let _ = sock.send(ClientEvent::DismissDialog(sn, pn)).await;
                                    }
                                },
                                "🚪 No, not yet"
                            }
                        }
                    }
                }
            }

            // Virtual D-pad — visible on touch screens, hidden on desktop (CSS media query).
            {
                let sn_up = SERVER_NAME();
                let pn_up = local_login_name_session();
                let sn_down = SERVER_NAME();
                let pn_down = local_login_name_session();
                let sn_left = SERVER_NAME();
                let pn_left = local_login_name_session();
                let sn_right = SERVER_NAME();
                let pn_right = local_login_name_session();
                let sn_int = SERVER_NAME();
                let pn_int = local_login_name_session();
                rsx! {
                    div { class: "ow-dpad",
                        // Row 1: only Up button (column 2)
                        div { class: "ow-dpad-empty" }
                        button {
                            class: "ow-dpad-btn",
                            tabindex: "-1",
                            onclick: move |_| {
                                let sn = sn_up.clone();
                                let pn = pn_up.clone();
                                let sock = socket_up.clone();
                                async move {
                                    let _ = sock.send(ClientEvent::MovePlayer(sn, pn, Direction::Up)).await;
                                }
                            },
                            "▲"
                        }
                        div { class: "ow-dpad-empty" }
                        // Row 2: Left, Interact, Right
                        button {
                            class: "ow-dpad-btn",
                            tabindex: "-1",
                            onclick: move |_| {
                                let sn = sn_left.clone();
                                let pn = pn_left.clone();
                                let sock = socket_left.clone();
                                async move {
                                    let _ = sock.send(ClientEvent::MovePlayer(sn, pn, Direction::Left)).await;
                                }
                            },
                            "◀"
                        }
                        button {
                            class: "ow-dpad-btn ow-dpad-center",
                            tabindex: "-1",
                            onclick: move |_| {
                                let sn = sn_int.clone();
                                let pn = pn_int.clone();
                                let sock = socket_interact.clone();
                                async move {
                                    let _ = sock.send(ClientEvent::Interact(sn, pn)).await;
                                }
                            },
                            "⚔"
                        }
                        button {
                            class: "ow-dpad-btn",
                            tabindex: "-1",
                            onclick: move |_| {
                                let sn = sn_right.clone();
                                let pn = pn_right.clone();
                                let sock = socket_right.clone();
                                async move {
                                    let _ = sock.send(ClientEvent::MovePlayer(sn, pn, Direction::Right)).await;
                                }
                            },
                            "▶"
                        }
                        // Row 3: only Down button (column 2)
                        div { class: "ow-dpad-empty" }
                        button {
                            class: "ow-dpad-btn",
                            tabindex: "-1",
                            onclick: move |_| {
                                let sn = sn_down.clone();
                                let pn = pn_down.clone();
                                let sock = socket_down.clone();
                                async move {
                                    let _ = sock.send(ClientEvent::MovePlayer(sn, pn, Direction::Down)).await;
                                }
                            },
                            "▼"
                        }
                        div { class: "ow-dpad-empty" }
                    }
                }
            }

            div { class: "ow-hud",
                span { class: "ow-map-name", "📍 {ow.map_id}" }
                span { class: "ow-controls", "Arrow keys / D-pad: move  |  Enter / ⚔: interact" }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| async move {
                        let _ = socket
                            .send(ClientEvent::ExitOverworld(SERVER_NAME()))
                            .await;
                    },
                    "⚔️ Back to Fight"
                }
            }
        }
    }
}

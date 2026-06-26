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

/// Emoji shown inside each tile so the map is readable at a glance.
fn tile_emoji(kind: &TileKind) -> &'static str {
    match kind {
        TileKind::Floor => "",
        TileKind::Wall => "🧱",
        TileKind::Grass => "",
        TileKind::Water => "💧",
        TileKind::Door { .. } => "🚪",
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

    rsx! {
        div {
            class: "ow-container",
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: move |e: KeyboardEvent| async move {
                // Compute strings fresh each invocation so the outer closure stays FnMut.
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

            // CSS Grid: tiles in normal flow (left→right, top→bottom); sprites overlay via position:absolute
            div {
                class: "ow-grid",
                // width constrains how many 48px tiles fit per row — height grows naturally
                style: "width: {ow.width * TILE_PX}px;",

                // Tiles in row-major order (flex-wrap breaks at the container width)
                for tile_kind in ow.tiles.iter().flatten() {
                    div {
                        class: "{tile_css(tile_kind)}",
                        style: "display:flex; align-items:center; justify-content:center; font-size:1.4rem; user-select:none;",
                        "{tile_emoji(tile_kind)}"
                    }
                }

                // NPC sprites
                for npc in ow.npcs.iter() {
                    div {
                        class: "ow-sprite ow-npc",
                        style: "left:{npc.pos.x * TILE_PX}px; top:{npc.pos.y * TILE_PX}px; width:{TILE_PX}px; height:{TILE_PX}px;",
                        "🧓"
                    }
                }

                // Hero sprites
                for (hero_id , pos) in ow.player_positions.iter() {
                    div {
                        key: "{hero_id}",
                        class: "ow-sprite ow-hero",
                        style: "left:{pos.x * TILE_PX}px; top:{pos.y * TILE_PX}px; width:{TILE_PX}px; height:{TILE_PX}px;",
                        "🧑"
                    }
                }
            }

            // NPC dialog box
            if !ow.active_dialog.is_empty() {
                div { class: "ow-dialog",
                    for line in ow.active_dialog.iter() {
                        p { class: "ow-dialog-line", "{line}" }
                    }
                }
            }

            // Map name + controls hint
            div { class: "ow-hud",
                span { class: "ow-map-name", "📍 {ow.map_id}" }
                span { class: "ow-controls", "Arrow keys: move  |  Enter / Space: interact" }
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

use dioxus::html::Key;
use dioxus::prelude::*;
use dioxus_i18n::t;
use lib_rpg::{
    common::overworld::{Direction, TileKind},
    server::server_manager::ServerData,
};

use crate::{
    common::{CtxAppLang, CtxOverworldZoom, PATH_IMG, SERVER_NAME},
    game_channel::GameChannel,
    websocket_handler::event::ClientEvent,
};

const TILE_PX: i32 = 48;

/// Zoom shown the first time a player opens the map, before they touch the
/// controls. Also `main.rs`'s default when nothing is stored yet.
pub const DEFAULT_ZOOM: f32 = 0.85;
/// Range the − / + buttons move within, and the step they move by.
const MIN_ZOOM: f32 = 0.4;
const MAX_ZOOM: f32 = 1.5;
const ZOOM_STEP: f32 = 0.1;
/// Every zoom the buttons can reach is a multiple of this, including the default
/// and both ends of the range.
const ZOOM_GRID: f32 = 0.05;

/// Moves the zoom one step and clamps it to the allowed range.
///
/// Snapped back onto the grid afterwards, because the result is persisted and fed
/// straight back in on the next press: in f32 `0.85 + 0.1 + 0.1` is `1.0500001`,
/// and left alone that error compounds for as long as the player keeps pressing.
pub fn step_zoom(current: f32, steps: i32) -> f32 {
    let stepped = current + steps as f32 * ZOOM_STEP;
    let snapped = (stepped / ZOOM_GRID).round() * ZOOM_GRID;
    snapped.clamp(MIN_ZOOM, MAX_ZOOM)
}

/// The percentage shown on the zoom overlay. Rounded, not truncated: `0.95` is
/// `0.94999999` in f32, which a cast would display as 94%.
pub fn zoom_percent(zoom: f32) -> u32 {
    (zoom * 100.0).round() as u32
}

const HERO_SPRITES: &[&str] = &[
    "heroes/tile_0084.png",
    "heroes/tile_0085.png",
    "heroes/tile_0087.png",
    "heroes/tile_0096.png",
    "heroes/tile_0097.png",
    "heroes/tile_0098.png",
];

const NPC_SPRITES: &[&str] = &[
    "npc/tile_0086.png",
    "npc/tile_0088.png",
    "npc/tile_0099.png",
    "npc/tile_0100.png",
];

const BOSS_SPRITES: &[&str] = &[
    "bosses/tile_0108.png",
    "bosses/tile_0109.png",
    "bosses/tile_0110.png",
    "bosses/tile_0111.png",
];

// Picks a stable sprite index from an ID by summing its bytes mod count.
fn sprite_idx(id: &str, count: usize) -> usize {
    id.bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize))
        % count
}

fn hero_sprite(id: &str) -> &'static str {
    HERO_SPRITES[sprite_idx(id, HERO_SPRITES.len())]
}

fn tile_css(kind: &TileKind, locked: bool) -> &'static str {
    match kind {
        TileKind::Floor => "ow-tile ow-floor",
        TileKind::Wall => "ow-tile ow-wall",
        TileKind::Grass => "ow-tile ow-grass",
        TileKind::Water => "ow-tile ow-water",
        TileKind::Door { .. } => {
            if locked {
                "ow-tile ow-door ow-locked"
            } else {
                "ow-tile ow-door"
            }
        }
    }
}

fn is_door(kind: &TileKind) -> bool {
    matches!(kind, TileKind::Door { .. })
}

fn tile_img(kind: &TileKind) -> &'static str {
    match kind {
        TileKind::Floor => "tile_floor.svg",
        TileKind::Wall => "tile_wall.svg",
        TileKind::Grass => "tile_grass.svg",
        TileKind::Water => "tile_water.svg",
        TileKind::Door { .. } => "tile_door.svg",
    }
}

fn npc_sprite_file(npc: &lib_rpg::server::overworld_manager::NpcState) -> &'static str {
    if npc.fight_scenario_id.is_some() {
        BOSS_SPRITES[sprite_idx(&npc.id, BOSS_SPRITES.len())]
    } else {
        NPC_SPRITES[sprite_idx(&npc.id, NPC_SPRITES.len())]
    }
}

#[component]
pub fn OverworldMap() -> Element {
    let socket = use_context::<GameChannel>();
    let server_data = use_context::<Signal<ServerData>>();
    let local_login_name_session = use_context::<Signal<String>>();
    let app_lang = use_context::<CtxAppLang>().0;

    // Persisted in `App()`, not here: this component is unmounted for the whole of a
    // fight and remounted on the way back to the map, so a zoom loaded per-mount
    // started over from the default every time — and the load it used went through a
    // server setting, which an offline session has no way to reach at all.
    let mut tile_zoom = use_context::<CtxOverworldZoom>().0;

    let ow_state = server_data().core_game_data.overworld.clone();
    let Some(ow) = ow_state else {
        return rsx! {
            div {
                class: "ow-container",
                style: "justify-content:center; align-items:center; min-height:300px;",
                p { style: "color:#7fff7f; font-size:1.2rem;", {t!("overworld-entering")} }
            }
        };
    };

    // Copy socket (it's `Copy`) + other context values for the D-pad closures (each needs its own copy).
    let socket_up = socket;
    let socket_down = socket;
    let socket_left = socket;
    let socket_right = socket;
    let socket_interact = socket;
    let socket_confirm_fight = socket;
    let socket_dismiss = socket;

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
                let lang = app_lang();
                let direction = match e.key() {
                    Key::ArrowUp => Some(Direction::Up),
                    Key::ArrowDown => Some(Direction::Down),
                    Key::ArrowLeft => Some(Direction::Left),
                    Key::ArrowRight => Some(Direction::Right),
                    Key::Character(ref s) if s.eq_ignore_ascii_case("w") => Some(Direction::Up),
                    Key::Character(ref s) if s.eq_ignore_ascii_case("s") => Some(Direction::Down),
                    Key::Character(ref s) if s.eq_ignore_ascii_case("a") => Some(Direction::Left),
                    Key::Character(ref s) if s.eq_ignore_ascii_case("d") => {
                        Some(Direction::Right)
                    }
                    _ => None,
                };
                if let Some(direction) = direction {
                    e.prevent_default();
                    let _ = socket
                        .send(ClientEvent::MovePlayer(server_name, player_name, direction, lang))
                        .await;
                    return;
                }
                match e.key() {
                    Key::Enter => {
                        e.prevent_default();
                        let _ = socket
                            .send(ClientEvent::Interact(server_name, player_name, lang))
                            .await;
                    }
                    Key::Character(s) if s == " " => {
                        e.prevent_default();
                        let _ = socket
                            .send(ClientEvent::Interact(server_name, player_name, lang))
                            .await;
                    }
                    _ => {}
                }
            },

            // Map area: scroll wrapper + zoom overlay anchored to its corner.
            div { class: "ow-map-area",
                div { class: "ow-grid-scroll",
                    div {
                        class: "ow-grid",
                        style: "width: {ow.width * TILE_PX}px; zoom: {tile_zoom()};",

                        for (y, row) in ow.tiles.iter().enumerate() {
                            for (x, tile_kind) in row.iter().enumerate() {
                                div {
                                    class: tile_css(
                                        tile_kind,
                                        is_door(tile_kind)
                                            && ow
                                                .locked_doors
                                                .contains(&format!("{}_{}", x, y)),
                                    ),
                                    style: "background-image: url('{PATH_IMG}/{tile_img(tile_kind)}');",
                                }
                            }
                        }

                        // Defeated boss NPCs are hidden.
                        for npc in ow.npcs.iter() {
                            if !npc.defeated {
                                img {
                                    src: "{PATH_IMG}/{npc_sprite_file(npc)}",
                                    class: "ow-sprite ow-npc",
                                    style: "left:{npc.pos.x * TILE_PX}px; top:{npc.pos.y * TILE_PX}px; width:{TILE_PX}px; height:{TILE_PX}px;",
                                    alt: "{npc.id}",
                                }
                            }
                        }

                        for (hero_id, pos) in ow.player_positions.iter() {
                            img {
                                key: "{hero_id}",
                                src: "{PATH_IMG}/{hero_sprite(hero_id)}",
                                class: "ow-sprite ow-hero",
                                style: "left:{pos.x * TILE_PX}px; top:{pos.y * TILE_PX}px; width:{TILE_PX}px; height:{TILE_PX}px;",
                                alt: "hero",
                            }
                        }
                    }
                }
                // Zoom overlay — floats over the top-right corner of the map.
                div { class: "ow-zoom-overlay",
                    button {
                        class: "ow-zoom-btn",
                        tabindex: "-1",
                        onclick: move |_| tile_zoom.set(step_zoom(tile_zoom(), -1)),
                        "−"
                    }
                    span { class: "ow-zoom-label", "{zoom_percent(tile_zoom())}%" }
                    button {
                        class: "ow-zoom-btn",
                        tabindex: "-1",
                        onclick: move |_| tile_zoom.set(step_zoom(tile_zoom(), 1)),
                        "+"
                    }
                }

                // Dialog overlays the bottom of the map.
                if !ow.active_dialog.is_empty() {
                    div { class: "ow-dialog",
                        for line in ow.active_dialog.iter() {
                            p { class: "ow-dialog-line", "{line}" }
                        }
                        if ow.pending_fight.is_some() {
                            p { class: "ow-dialog-question", {t!("overworld-start-fight-question")} }
                            div { class: "ow-dialog-actions",
                                button {
                                    class: "ow-dialog-btn ow-dialog-btn-yes",
                                    tabindex: "-1",
                                    onclick: move |_| {
                                        let sn = SERVER_NAME();
                                        let pn = local_login_name_session();
                                        let lang = app_lang();
                                        let sock = socket_confirm_fight;
                                        async move {
                                            let _ = sock
                                                .send(ClientEvent::Interact(sn, pn, lang))
                                                .await;
                                        }
                                    },
                                    {t!("overworld-yes-fight")}
                                }
                                button {
                                    class: "ow-dialog-btn ow-dialog-btn-no",
                                    tabindex: "-1",
                                    onclick: move |_| {
                                        let sn = SERVER_NAME();
                                        let pn = local_login_name_session();
                                        let sock = socket_dismiss;
                                        async move {
                                            let _ = sock.send(ClientEvent::DismissDialog(sn, pn)).await;
                                        }
                                    },
                                    {t!("overworld-no-not-yet")}
                                }
                            }
                        }
                    }
                }

            } // ow-map-area

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
                                let lang = app_lang();
                                let sock = socket_up;
                                async move {
                                    let _ = sock
                                        .send(ClientEvent::MovePlayer(sn, pn, Direction::Up, lang))
                                        .await;
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
                                let lang = app_lang();
                                let sock = socket_left;
                                async move {
                                    let _ = sock
                                        .send(ClientEvent::MovePlayer(sn, pn, Direction::Left, lang))
                                        .await;
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
                                let lang = app_lang();
                                let sock = socket_interact;
                                async move {
                                    let _ = sock.send(ClientEvent::Interact(sn, pn, lang)).await;
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
                                let lang = app_lang();
                                let sock = socket_right;
                                async move {
                                    let _ = sock
                                        .send(ClientEvent::MovePlayer(sn, pn, Direction::Right, lang))
                                        .await;
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
                                let lang = app_lang();
                                let sock = socket_down;
                                async move {
                                    let _ = sock
                                        .send(ClientEvent::MovePlayer(sn, pn, Direction::Down, lang))
                                        .await;
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
                span { class: "ow-controls", {t!("overworld-controls-hint")} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_moves_by_ten_percent() {
        assert_eq!(zoom_percent(step_zoom(DEFAULT_ZOOM, -1)), 75);
        assert_eq!(zoom_percent(step_zoom(DEFAULT_ZOOM, 1)), 95);
    }

    #[test]
    fn stepping_stays_inside_the_allowed_range() {
        assert_eq!(
            zoom_percent(step_zoom(MIN_ZOOM, -1)),
            zoom_percent(MIN_ZOOM)
        );
        assert_eq!(zoom_percent(step_zoom(MAX_ZOOM, 1)), zoom_percent(MAX_ZOOM));
    }

    /// The zoom is persisted and read straight back in on the next press, so any
    /// error a step introduces is error the player keeps and compounds.
    #[test]
    fn repeated_steps_do_not_drift() {
        let mut zoom = DEFAULT_ZOOM;
        for _ in 0..6 {
            zoom = step_zoom(zoom, 1);
        }
        for _ in 0..6 {
            zoom = step_zoom(zoom, -1);
        }
        assert_eq!(
            zoom_percent(zoom),
            zoom_percent(DEFAULT_ZOOM),
            "a round trip must land back where it started"
        );
    }

    /// Walks every position the + button can reach and checks what the overlay
    /// would print at each one.
    #[test]
    fn every_reachable_zoom_reads_as_a_whole_step() {
        let mut zoom = MIN_ZOOM;
        let mut seen = vec![zoom_percent(zoom)];
        while zoom_percent(zoom) < zoom_percent(MAX_ZOOM) {
            zoom = step_zoom(zoom, 1);
            seen.push(zoom_percent(zoom));
        }
        assert_eq!(
            seen,
            vec![40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150],
            "no position may read back as 94% or 105% from f32 error"
        );
    }

    /// `main.rs` seeds the stored setting with `DEFAULT_ZOOM`, so it has to be a
    /// position the buttons can actually return to.
    #[test]
    fn the_default_is_on_the_grid() {
        assert_eq!(
            zoom_percent(step_zoom(DEFAULT_ZOOM, 0)),
            zoom_percent(DEFAULT_ZOOM)
        );
        assert_eq!(
            zoom_percent(step_zoom(step_zoom(DEFAULT_ZOOM, 1), -1)),
            zoom_percent(DEFAULT_ZOOM)
        );
        assert!((MIN_ZOOM..=MAX_ZOOM).contains(&DEFAULT_ZOOM));
    }
}

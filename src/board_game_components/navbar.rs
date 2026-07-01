use crate::{
    common::DISCONNECTED_USER,
    websocket_handler::{NO_CLIENT_ID, msg_from_client::send_disconnect_from_server_data},
};
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::{
    auth_manager::server_fn::logout,
    common::{ADMIN, Route},
    components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
            AlertDialogRoot, AlertDialogTitle,
        },
        button::{Button, ButtonVariant},
    },
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::send_disconnect_from_server_data as send_quit,
    },
};

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    let server_data = use_context::<Signal<ServerData>>();

    // nav
    let navigator = use_navigator();

    // dialog open states — lifted here so the roots can live outside the navbar div
    let mut help_open = use_signal(|| false);
    let mut quit_open = use_signal(|| false);

    // snapshot
    let snap_local_login_name_session = local_login_name_session();

    rsx! {
        div { class: "page-layout",
            // ── Navbar bar ────────────────────────────────────────────────────────
            div { class: "navbar",
                // Left: brand + admin panel link
                div { style: "display: flex; align-items: center; gap: 1rem;",
                    Link {
                        class: "navbar-brand",
                        to: Route::Home {},
                        onclick: move |_| async move {
                            send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                        },
                        "⚔️ RPG"
                    }
                    if snap_local_login_name_session == ADMIN.to_string() {
                        Link {
                            class: "navbar-admin-link",
                            to: Route::AdminPage {},
                            "🛡️ Panel"
                        }
                    }
                }
                // Right: trigger buttons only (no dialog roots here)
                div { style: "display: flex; flex-direction: row; align-items: center; gap: 0.75rem;",
                    // Help trigger
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| help_open.set(true),
                        "?"
                    }
                    // Quit-game trigger (only while a game is running)
                    if server_data().core_game_data.game_phase == GamePhase::Running {
                        Button {
                            onclick: move |_| quit_open.set(true),
                            r#type: "button",
                            "Quit game"
                        }
                    }
                    if snap_local_login_name_session != *DISCONNECTED_USER {
                        span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                    }
                    Button {
                        variant: if snap_local_login_name_session == *DISCONNECTED_USER { ButtonVariant::Secondary } else { ButtonVariant::Destructive },
                        onclick: move |_| async move {
                            if local_login_name_session() != *DISCONNECTED_USER {
                                match logout().await {
                                    Ok(_) => {
                                        tracing::info!("{} is logged out", local_login_name_session());
                                        let _ = socket
                                            .clone()
                                            .send(ClientEvent::RequestLogOut(local_login_name_session()))
                                            .await;
                                        *local_login_name_session.write() = (*DISCONNECTED_USER).to_string();
                                        *local_login_id_session.write() = NO_CLIENT_ID;
                                    }
                                    Err(_) => {
                                        tracing::info!("Error on {} logout", local_login_name_session())
                                    }
                                }
                            }
                            navigator.push(Route::Home {});
                        },
                        if snap_local_login_name_session == *DISCONNECTED_USER {
                            "Sign in"
                        } else {
                            "Sign out"
                        }
                    }
                }
            }

            // ── Dialog roots — rendered at layout level, NOT inside the navbar div ──

            // Help dialog
            AlertDialogRoot { open: help_open(), on_open_change: move |v| help_open.set(v),
                AlertDialogContent {
                    AlertDialogTitle { "How to play" }
                    AlertDialogDescription {
                        div { style: "text-align:left; line-height:1.8; max-height:70vh; overflow-y:auto; padding-right:4px;",
                            // Getting started
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-bottom:2px;",
                                "🚀 Getting started"
                            }
                            p { "1. 🔐 Sign in or create an account on the login page." }
                            p { "2. 🌍 Choose a universe (LOTR or Pokémon) when creating a server." }
                            p { "3. 🎮 From the home page, create a new game or join an ongoing one." }

                            // Game mode
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "🕹️ Game modes"
                            }
                            p {
                                "• Multiplayer — each connected player picks exactly one hero; other cards are locked 🔒 for other players."
                            }
                            p {
                                "• Single-player — one player picks multiple heroes and controls them all in battle; click a selected card again to deselect it."
                            }

                            // Lobby & character selection
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "🧙 Lobby & character selection"
                            }
                            p {
                                "4. Select your character card in the lobby. Wait for all players to be ready."
                            }
                            p { "5. ▶️ The host clicks 'Start Game' once everyone has chosen." }

                            // Combat
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "⚔️ Combat"
                            }
                            p {
                                "6. On your turn, click ⚔️ on your character card to open the attack list, then pick an attack."
                            }
                            p {
                                "7. 🎯 Click target buttons to select your target(s), then confirm with '⚔️ Launch Attack'."
                            }
                            p {
                                "8. 💊 Click 💊 on your character card to use a potion (counts as your turn action)."
                            }

                            // Toolbar
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "🛠️ Game toolbar"
                            }
                            p { "9. 📦 Inventory — view your hero's stats and equipment." }
                            p {
                                "10. 📊 Stats — track damage dealt, healing done, kill count, and scenario progress bar."
                            }
                            p {
                                "11. 📜 Scenarios — side sheet listing all stages with their completion status (Not Started / In Progress / ✅ Done)."
                            }
                            p {
                                "12. ⚙️ Settings — toggle 'Attack Tooltips' to show/hide attack descriptions on hover."
                            }

                            // Overworld
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "🗺️ Overworld exploration"
                            }
                            p {
                                "13. The host clicks '🗺 Overworld' to enter the tile-map exploration mode."
                            }
                            p { "    • Arrow keys / D-pad — move your hero." }
                            p { "    • Enter or Space — interact with adjacent NPCs." }
                            p {
                                "    • Walking on grass may trigger a random encounter (50 % chance per step)."
                            }
                            p {
                                "    • Interact with a boss NPC to start its pre-fight dialog, then confirm to begin the fight."
                            }
                            p {
                                "    • Defeating a boss NPC unlocks the next door and removes the NPC from the map."
                            }
                            p { "    • '⚔️ Back to Fight' returns to the active fight at any time." }

                            // Store
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "🛒 Store"
                            }
                            p {
                                "14. The store opens between scenarios (end-of-scenario screen) via the '🛒 Shop' button."
                            }
                            p {
                                "    • Equipment tab — weapons, armour, rings and more; bought items go to your Bag."
                            }
                            p { "    • Consumables tab — potions (HP / Mana / Vigor / Berserk / Resurrection)." }
                            p {
                                "    • Bag tab — sell unequipped items for 50 % of their price; equip them from the Inventory sheet."
                            }
                            p { "    • Gold is earned as loot at the end of each scenario." }

                            // Progression
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "🏆 Progression"
                            }
                            p { "15. At the end of a scenario the host loads the next stage." }
                            p {
                                "16. Each universe has 10 progressive stages. Complete them all to win!"
                            }
                            p {
                                "17. Save slots (up to 3 by default) let you continue a run later from the Load Game page."
                            }
                            p {
                                "18. ⚙️ Settings — toggle auto-save, attack tooltips, boss HP / energy bars, and hero aggro display."
                            }

                            // Admin
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                "🛡️ Admin panel"
                            }
                            p { "19. If you are an admin, access the 🛡️ Panel link in the navbar." }
                            p { "    • Users tab: manage accounts and connection status." }
                            p { "    • Characters tab: browse all heroes and bosses by universe." }
                            p {
                                "    • Scenarios tab: add, edit or delete scenarios via inline JSON editor."
                            }
                        }
                    }
                    AlertDialogAction {
                        AlertDialogCancel { "Close" }
                    }
                }
            }

            // Quit-game confirmation dialog
            AlertDialogRoot { open: quit_open(), on_open_change: move |v| quit_open.set(v),
                AlertDialogContent {
                    AlertDialogTitle { "Quit Game" }
                    AlertDialogDescription { "Are you sure you want to quit the game?" }
                    AlertDialogAction {
                        AlertDialogCancel { "Cancel" }
                        AlertDialogAction {
                            on_click: move |_| {
                                async move {
                                    send_quit(socket, &local_login_name_session()).await;
                                    let navigator = use_navigator();
                                    navigator.push(Route::Home {});
                                }
                            },
                            "Confirm"
                        }
                    }
                }
            }

            Outlet::<Route> {}

            // ── Footer ────────────────────────────────────────────────────────────
            footer { class: "app-footer",
                div { class: "app-footer-inner",
                    // Brand
                    div { class: "app-footer-brand",
                        span { class: "app-footer-icon", "⚔️" }
                        span { class: "app-footer-name", "dx-rpg" }
                        span { class: "app-footer-version", {concat!("v", env!("CARGO_PKG_VERSION"))} }
                    }
                    // About
                    div { class: "app-footer-section",
                        span { class: "app-footer-section-title", "About" }
                        a {
                            href: "https://github.com/r0nd0ud0u/dx-rpg",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "GitHub"
                        }
                        span { class: "app-footer-sep", "·" }
                        a {
                            href: "https://github.com/r0nd0ud0u/lib-rpg",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "lib-rpg engine"
                        }
                        span { class: "app-footer-sep", "·" }
                        a {
                            href: "https://dioxuslabs.com",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Built with Dioxus"
                        }
                        span { "⚡ Rust + WASM" }
                    }
                    // Contact
                    div { class: "app-footer-section",
                        span { class: "app-footer-section-title", "Contact" }
                        a {
                            href: "https://github.com/r0nd0ud0u/dx-rpg/issues",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Report an issue"
                        }
                        span { class: "app-footer-sep", "·" }
                        a {
                            href: "https://github.com/r0nd0ud0u/dx-rpg/discussions",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "Discussions"
                        }
                    }
                }
            }
        } // end page-layout
    }
}

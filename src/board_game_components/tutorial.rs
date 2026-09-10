//! The "How to play" dialog, reachable from the ❓ navbar button and opened once
//! automatically on a device that has never seen it (see `CtxTutorialSeen`).
//!
//! Split into tabs rather than one scrolling manual: a player who opens it mid-fight
//! wants the combat rules, not the sign-in steps, and `default_tab` picks that tab from
//! the game phase so the answer is already on screen.

use dioxus::prelude::*;
use dioxus_i18n::t;
use lib_rpg::server::server_manager::GamePhase;

use crate::components::{
    alert_dialog::{
        AlertDialogActions, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
        AlertDialogRoot, AlertDialogTitle,
    },
    tabs::{TabContent, TabList, TabTrigger, Tabs},
};

const TAB_BASICS: &str = "basics";
const TAB_COMBAT: &str = "combat";
const TAB_WORLD: &str = "world";
const TAB_GEAR: &str = "gear";
const TAB_PROGRESS: &str = "progress";
const TAB_PRO: &str = "pro";
const TAB_ADMIN: &str = "admin";

/// Tab to land on, from what the player is doing when they hit ❓.
fn default_tab(phase: &GamePhase) -> &'static str {
    match phase {
        GamePhase::Running => TAB_COMBAT,
        GamePhase::Overworld => TAB_WORLD,
        _ => TAB_BASICS,
    }
}

/// One numbered card in the "First steps" tab.
#[component]
fn HelpStep(number: usize, title: String, body: String) -> Element {
    rsx! {
        div { class: "help-step",
            span { class: "help-step-number", "{number}" }
            div {
                p { class: "help-step-title", "{title}" }
                p { class: "help-step-body", "{body}" }
            }
        }
    }
}

/// A titled paragraph — the building block of every tab but "First steps".
#[component]
fn HelpBlock(title: String, body: String) -> Element {
    rsx! {
        div { class: "help-block",
            p { class: "help-block-title", "{title}" }
            p { class: "help-block-body", "{body}" }
        }
    }
}

/// Highlighted aside: the offline shortcut, the admin intro, the overworld lead-in.
#[component]
fn HelpCallout(title: Option<String>, body: String) -> Element {
    rsx! {
        div { class: "help-callout",
            if let Some(title) = title {
                p { class: "help-callout-title", "{title}" }
            }
            p { class: "help-callout-body", "{body}" }
        }
    }
}

/// Bulleted line — icon legends, overworld rules, pro tips.
#[component]
fn HelpBullet(body: String) -> Element {
    rsx! {
        p { class: "help-bullet", "{body}" }
    }
}

/// Control name on the left, the keys that fire it on the right.
#[component]
fn HelpKeys(label: String, keys: String) -> Element {
    rsx! {
        div { class: "help-keys-row",
            span { class: "help-keys-label", "{label}" }
            span { class: "help-keys-value", "{keys}" }
        }
    }
}

/// "How to play". `phase` only picks the opening tab; `is_admin` hides the admin tab
/// from the players it would mean nothing to; `first_run` adds the line telling a player
/// who did not ask for this dialog how to get it back.
#[component]
pub fn HowToPlayDialog(
    open: Signal<bool>,
    is_admin: bool,
    phase: GamePhase,
    first_run: bool,
) -> Element {
    let mut open_signal = open;
    rsx! {
        AlertDialogRoot { open: open(), on_open_change: move |v| open_signal.set(v),
            AlertDialogContent { class: "help-dialog".to_owned(),
                AlertDialogTitle { {t!("help-title")} }
                AlertDialogDescription { {t!("help-intro")} }
                if first_run {
                    p { class: "help-note", {t!("help-first-run-hint")} }
                }
                // Remounted on every open (rather than left mounted and hidden) so
                // `default_value` re-reads the phase each time — see `default_tab`.
                if open() {
                    div { class: "help-body",
                        Tabs {
                            class: "help-tabs".to_owned(),
                            default_value: default_tab(&phase).to_owned(),
                            horizontal: true,
                            TabList {
                                TabTrigger {
                                    value: TAB_BASICS.to_owned(),
                                    index: 0_usize,
                                    {t!("help-tab-basics")}
                                }
                                TabTrigger {
                                    value: TAB_COMBAT.to_owned(),
                                    index: 1_usize,
                                    {t!("help-tab-combat")}
                                }
                                TabTrigger {
                                    value: TAB_WORLD.to_owned(),
                                    index: 2_usize,
                                    {t!("help-tab-world")}
                                }
                                TabTrigger {
                                    value: TAB_GEAR.to_owned(),
                                    index: 3_usize,
                                    {t!("help-tab-gear")}
                                }
                                TabTrigger {
                                    value: TAB_PROGRESS.to_owned(),
                                    index: 4_usize,
                                    {t!("help-tab-progress")}
                                }
                                TabTrigger { value: TAB_PRO.to_owned(), index: 5_usize,
                                    {t!("help-tab-pro")}
                                }
                                if is_admin {
                                    TabTrigger {
                                        value: TAB_ADMIN.to_owned(),
                                        index: 6_usize,
                                        {t!("help-tab-admin")}
                                    }
                                }
                            }
                            TabContent { value: TAB_BASICS.to_owned(), index: 0_usize,
                                div { class: "help-panel", BasicsTab {} }
                            }
                            TabContent { value: TAB_COMBAT.to_owned(), index: 1_usize,
                                div { class: "help-panel", CombatTab {} }
                            }
                            TabContent { value: TAB_WORLD.to_owned(), index: 2_usize,
                                div { class: "help-panel", WorldTab {} }
                            }
                            TabContent { value: TAB_GEAR.to_owned(), index: 3_usize,
                                div { class: "help-panel", GearTab {} }
                            }
                            TabContent {
                                value: TAB_PROGRESS.to_owned(),
                                index: 4_usize,
                                div { class: "help-panel", ProgressTab {} }
                            }
                            TabContent { value: TAB_PRO.to_owned(), index: 5_usize,
                                div { class: "help-panel", ProTab {} }
                            }
                            if is_admin {
                                TabContent {
                                    value: TAB_ADMIN.to_owned(),
                                    index: 6_usize,
                                    div { class: "help-panel", AdminTab {} }
                                }
                            }
                        }
                    }
                }
                AlertDialogActions {
                    AlertDialogCancel { {t!("common-close")} }
                }
            }
        }
    }
}

#[component]
fn BasicsTab() -> Element {
    rsx! {
        HelpStep {
            number: 1_usize,
            title: t!("help-basics-1-title"),
            body: t!("help-basics-1-body"),
        }
        HelpStep {
            number: 2_usize,
            title: t!("help-basics-2-title"),
            body: t!("help-basics-2-body"),
        }
        HelpStep {
            number: 3_usize,
            title: t!("help-basics-3-title"),
            body: t!("help-basics-3-body"),
        }
        p { class: "help-note", {t!("help-basics-host-note")} }
        HelpCallout {
            title: t!("help-basics-offline-title"),
            body: t!("help-basics-offline-body"),
        }
        p { class: "help-section-title", {t!("help-basics-legend-title")} }
        div { class: "help-legend",
            HelpBullet { body: t!("help-legend-attack") }
            HelpBullet { body: t!("help-legend-potion") }
            HelpBullet { body: t!("help-legend-aggro") }
            HelpBullet { body: t!("help-legend-extra") }
            HelpBullet { body: t!("help-legend-taken") }
            HelpBullet { body: t!("help-legend-badge") }
        }
    }
}

#[component]
fn CombatTab() -> Element {
    rsx! {
        HelpBlock {
            title: t!("help-combat-flow-title"),
            body: t!("help-combat-flow-body"),
        }
        HelpBlock {
            title: t!("help-combat-potion-title"),
            body: t!("help-combat-potion-body"),
        }
        HelpBlock {
            title: t!("help-combat-energy-title"),
            body: t!("help-combat-energy-body"),
        }
        HelpBlock {
            title: t!("help-combat-order-title"),
            body: t!("help-combat-order-body"),
        }
        HelpBlock {
            title: t!("help-combat-crit-title"),
            body: t!("help-combat-crit-body"),
        }
        HelpBlock {
            title: t!("help-combat-read-title"),
            body: t!("help-combat-read-body"),
        }
    }
}

#[component]
fn WorldTab() -> Element {
    rsx! {
        HelpCallout { body: t!("help-world-intro") }
        div { class: "help-keys",
            HelpKeys {
                label: t!("help-world-move-label"),
                keys: t!("help-world-move-keys"),
            }
            HelpKeys {
                label: t!("help-world-interact-label"),
                keys: t!("help-world-interact-keys"),
            }
            HelpKeys {
                label: t!("help-world-zoom-label"),
                keys: t!("help-world-zoom-keys"),
            }
        }
        HelpBullet { body: t!("help-world-grass") }
        HelpBullet { body: t!("help-world-boss") }
        HelpBullet { body: t!("help-world-door") }
        HelpBullet { body: t!("help-world-back") }
    }
}

#[component]
fn GearTab() -> Element {
    rsx! {
        HelpBlock {
            title: t!("help-gear-store-title"),
            body: t!("help-gear-store-body"),
        }
        HelpBlock {
            title: t!("help-gear-tabs-title"),
            body: t!("help-gear-tabs-body"),
        }
        HelpBlock {
            title: t!("help-gear-equip-title"),
            body: t!("help-gear-equip-body"),
        }
        HelpBlock {
            title: t!("help-gear-gold-title"),
            body: t!("help-gear-gold-body"),
        }
    }
}

#[component]
fn ProgressTab() -> Element {
    rsx! {
        HelpBlock {
            title: t!("help-progress-scenario-title"),
            body: t!("help-progress-scenario-body"),
        }
        HelpBlock {
            title: t!("help-progress-talents-title"),
            body: t!("help-progress-talents-body"),
        }
        HelpBlock {
            title: t!("help-progress-cap-title"),
            body: t!("help-progress-cap-body"),
        }
        HelpBlock {
            title: t!("help-progress-stages-title"),
            body: t!("help-progress-stages-body"),
        }
        HelpBlock {
            title: t!("help-progress-save-title"),
            body: t!("help-progress-save-body"),
        }
    }
}

#[component]
fn ProTab() -> Element {
    rsx! {
        p { class: "help-section-title", {t!("help-pro-title")} }
        div { class: "help-legend",
            HelpBullet { body: t!("help-pro-speed") }
            HelpBullet { body: t!("help-pro-panel") }
            HelpBullet { body: t!("help-pro-aggro") }
            HelpBullet { body: t!("help-pro-ultimate") }
            HelpBullet { body: t!("help-pro-berserker") }
            HelpBullet { body: t!("help-pro-mystery") }
            HelpBullet { body: t!("help-pro-save") }
            HelpBullet { body: t!("help-pro-solo") }
        }
    }
}

#[component]
fn AdminTab() -> Element {
    rsx! {
        HelpCallout { body: t!("help-admin-intro") }
        HelpBullet { body: t!("help-admin-users") }
        HelpBullet { body: t!("help-admin-characters") }
        HelpBullet { body: t!("help-admin-scenarios") }
        HelpBullet { body: t!("help-admin-content") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_tab_follows_the_game_phase() {
        assert_eq!(TAB_COMBAT, default_tab(&GamePhase::Running));
        assert_eq!(TAB_WORLD, default_tab(&GamePhase::Overworld));
        assert_eq!(TAB_BASICS, default_tab(&GamePhase::Default));
        assert_eq!(TAB_BASICS, default_tab(&GamePhase::InitGame));
        assert_eq!(TAB_BASICS, default_tab(&GamePhase::Loading));
        assert_eq!(TAB_BASICS, default_tab(&GamePhase::Ended));
    }
}

use dioxus::prelude::*;
use lib_rpg::{
    character_mod::{attack_type::AtksInfo, stats_in_game::StatsInGame},
    server::server_manager::ServerData,
};

use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};

// ─── helpers ────────────────────────────────────────────────────────────────

fn total_real_dmg(atks: &[AtksInfo]) -> i64 {
    atks.iter()
        .flat_map(|a| a.totals_by_target.values())
        .map(|t| t.total_real_dmg.abs())
        .sum()
}

fn total_real_heal(atks: &[AtksInfo]) -> i64 {
    atks.iter()
        .flat_map(|a| a.totals_by_target.values())
        .map(|t| t.total_real_heal)
        .sum()
}

fn total_uses(atks: &[AtksInfo]) -> i64 {
    atks.iter().map(|a| a.nb_use).sum()
}

// ─── Summary cards ──────────────────────────────────────────────────────────

#[component]
fn StatCard(
    label: String,
    value: String,
    color: String,
    icon: String,
    #[props(default)] wide: bool,
) -> Element {
    let card_class = if wide {
        "rpg-stat-card rpg-stat-card--wide"
    } else {
        "rpg-stat-card"
    };
    rsx! {
        div { class: "{card_class}", style: "border-left: 4px solid {color};",
            span { class: "rpg-stat-card-icon", "{icon}" }
            div { class: "rpg-stat-card-body",
                span { class: "rpg-stat-card-value", "{value}" }
                span { class: "rpg-stat-card-label", "{label}" }
            }
        }
    }
}

// ─── Horizontal bar ─────────────────────────────────────────────────────────

#[component]
fn ProgressBar(label: String, value: i64, max: i64, color: String, suffix: String) -> Element {
    let pct = if max == 0 {
        0.0_f64
    } else {
        (value as f64 / max as f64 * 100.0).min(100.0)
    };
    rsx! {
        div { class: "rpg-bar-row",
            span { class: "rpg-bar-label", "{label}" }
            div { class: "rpg-bar-track",
                div {
                    class: "rpg-bar-fill",
                    style: "width: {pct:.1}%; background: {color};",
                }
            }
            span { class: "rpg-bar-value", "{value}{suffix}" }
        }
    }
}

// ─── Attack-usage frequency chart ───────────────────────────────────────────

#[component]
fn AtkUsageChart(atks_info: Vec<AtksInfo>) -> Element {
    let total_uses: i64 = atks_info.iter().map(|a| a.nb_use).sum();
    if total_uses == 0 {
        return rsx! {
            p { class: "rpg-no-data", "No attacks recorded yet." }
        };
    }

    // sort descending by usage
    let mut sorted = atks_info.clone();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.nb_use));
    let colors = [
        "var(--rpg-gold)",
        "var(--rpg-teal)",
        "var(--secondary-color-2)",
        "var(--secondary-success-color)",
        "#9b59b6",
        "#e67e22",
        "#1abc9c",
        "#e74c3c",
    ];

    rsx! {
        div { class: "rpg-section",
            h4 { class: "rpg-section-title", "⚔️ Attack Frequency" }
            div { class: "rpg-bar-list",
                for (i, atk) in sorted.iter().enumerate() {
                    ProgressBar {
                        label: format!("{} (×{})", atk.atk_name, atk.nb_use),
                        value: atk.nb_use,
                        max: sorted.first().map(|a| a.nb_use).unwrap_or(1),
                        color: colors[i % colors.len()].to_owned(),
                        suffix: "".to_owned(),
                    }
                }
            }
        }
    }
}

// ─── Damage/heal per attack ──────────────────────────────────────────────────

#[component]
fn AtkAmountTable(atks_info: Vec<AtksInfo>) -> Element {
    let has_data = atks_info.iter().any(|a| !a.totals_by_target.is_empty());
    if !has_data {
        return rsx! {
            p { class: "rpg-no-data", "No damage or heal data yet." }
        };
    }

    let max_dmg = atks_info
        .iter()
        .flat_map(|a| a.totals_by_target.values())
        .map(|t| t.total_real_dmg.abs())
        .max()
        .unwrap_or(1);
    let max_heal = atks_info
        .iter()
        .flat_map(|a| a.totals_by_target.values())
        .map(|t| t.total_real_heal)
        .max()
        .unwrap_or(1);

    rsx! {
        div { class: "rpg-section",
            h4 { class: "rpg-section-title", "🗡️ Damage dealt" }
            div { class: "rpg-bar-list",
                for atk in atks_info
                    .iter()
                    .filter(|a| a.totals_by_target.values().any(|t| t.total_real_dmg < 0))
                {
                    for (target, totals) in atk.totals_by_target.iter().filter(|(_, t)| t.total_real_dmg < 0) {
                        ProgressBar {
                            label: format!("{} → {}", atk.atk_name, target),
                            value: totals.total_real_dmg.abs(),
                            max: max_dmg.max(1),
                            color: "var(--secondary-color-2)".to_owned(),
                            suffix: " dmg".to_owned(),
                        }
                    }
                }
            }
        }
        div { class: "rpg-section",
            h4 { class: "rpg-section-title", "💚 Healing done" }
            div { class: "rpg-bar-list",
                for atk in atks_info
                    .iter()
                    .filter(|a| a.totals_by_target.values().any(|t| t.total_real_heal > 0))
                {
                    for (target, totals) in atk.totals_by_target.iter().filter(|(_, t)| t.total_real_heal > 0) {
                        ProgressBar {
                            label: format!("{} → {}", atk.atk_name, target),
                            value: totals.total_real_heal,
                            max: max_heal.max(1),
                            color: "var(--secondary-success-color)".to_owned(),
                            suffix: " hp".to_owned(),
                        }
                    }
                }
            }
        }
    }
}

// ─── Full hero/party stats panel ────────────────────────────────────────────

#[component]
fn HeroStatsPanel(atks_info: Vec<AtksInfo>, rounds: usize) -> Element {
    let dmg = total_real_dmg(&atks_info);
    let heal = total_real_heal(&atks_info);
    let uses = total_uses(&atks_info);
    let dpr = if rounds > 0 { dmg / rounds as i64 } else { dmg };
    let hpr = if rounds > 0 {
        heal / rounds as i64
    } else {
        heal
    };
    let top_atk = atks_info
        .iter()
        .max_by_key(|a| a.nb_use)
        .map(|a| a.atk_name.as_str())
        .unwrap_or("—");

    rsx! {
        div { class: "rpg-stats-panel",
            // ── Summary cards ──────────────────────────────────────────────
            div { class: "rpg-stat-cards",
                StatCard {
                    label: "Total Damage".to_owned(),
                    value: format!("{dmg}"),
                    color: "var(--secondary-color-2)".to_owned(),
                    icon: "🗡️".to_owned(),
                }
                StatCard {
                    label: "Total Heal".to_owned(),
                    value: format!("{heal}"),
                    color: "var(--secondary-success-color)".to_owned(),
                    icon: "💚".to_owned(),
                }
                StatCard {
                    label: "Dmg / Round".to_owned(),
                    value: format!("{dpr}"),
                    color: "var(--rpg-gold)".to_owned(),
                    icon: "📈".to_owned(),
                }
                StatCard {
                    label: "Heal / Round".to_owned(),
                    value: format!("{hpr}"),
                    color: "var(--rpg-teal)".to_owned(),
                    icon: "✨".to_owned(),
                }
                StatCard {
                    label: "Attacks cast".to_owned(),
                    value: format!("{uses}"),
                    color: "#9b59b6".to_owned(),
                    icon: "🔢".to_owned(),
                }
                StatCard {
                    label: "Favourite".to_owned(),
                    value: top_atk.to_owned(),
                    color: "#e67e22".to_owned(),
                    icon: "⭐".to_owned(),
                    wide: true,
                }
            }

            // ── Charts ─────────────────────────────────────────────────────
            AtkUsageChart { atks_info: atks_info.clone() }
            AtkAmountTable { atks_info }
        }
    }
}

// ─── Main TabStats component ─────────────────────────────────────────────────

#[component]
pub fn TabStats() -> Element {
    let server_data = use_context::<Signal<ServerData>>();

    let server_data_snap = server_data();
    let game_state = &server_data_snap.core_game_data.game_manager.game_state;
    let heroes = server_data_snap
        .core_game_data
        .game_manager
        .pm
        .active_heroes
        .clone();
    let rounds = game_state.current_turn_nb;

    // Merged stats for "All" tab
    let all_atk_info: Vec<AtksInfo> = {
        let mut merged: Vec<AtksInfo> = Vec::new();
        for hero in &heroes {
            let hero_info = game_state
                .stats_in_game
                .get(&hero.id_name)
                .cloned()
                .unwrap_or_default();
            for atk_info in hero_info.all_atk_info {
                if let Some(existing) = merged.iter_mut().find(|a| a.atk_name == atk_info.atk_name)
                {
                    existing.nb_use += atk_info.nb_use;
                    for (target, totals) in &atk_info.totals_by_target {
                        let entry = existing.totals_by_target.entry(target.clone()).or_default();
                        entry.total_full_heal += totals.total_full_heal;
                        entry.total_real_heal += totals.total_real_heal;
                        entry.total_full_dmg += totals.total_full_dmg;
                        entry.total_real_dmg += totals.total_real_dmg;
                    }
                } else {
                    merged.push(atk_info);
                }
            }
        }
        merged
    };

    rsx! {
        Tabs {
            default_value: "tab0".to_owned(),
            horizontal: true,
            max_width: "42em",
            TabList {
                TabTrigger { value: "tab0".to_owned(), index: 0_usize, "🌐 Party" }
                for (i, c) in heroes.clone().into_iter().enumerate() {
                    TabTrigger { value: format!("tab{}", i + 1), index: i + 1, "⚔️ {c.db_full_name}" }
                }
            }
            TabContent { value: "tab0".to_owned(), index: 0_usize, width: "42em",
                HeroStatsPanel { atks_info: all_atk_info, rounds }
            }
            for (i, c) in heroes.into_iter().enumerate() {
                TabContent {
                    value: format!("tab{}", i + 1),
                    index: i + 1,
                    width: "42em",
                    HeroStatsPanel {
                        atks_info: game_state
                            .stats_in_game
                            .get(&c.id_name)
                            .unwrap_or(&StatsInGame::default())
                            .all_atk_info
                            .clone(),
                        rounds,
                    }
                }
            }
        }
    }
}

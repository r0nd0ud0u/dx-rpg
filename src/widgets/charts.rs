use dioxus::prelude::*;
use dioxus_charts::BarChart;
use lib_rpg::{
    character_mod::{
        attack_type::AtksInfo,
        stats_in_game::{StatsInGame, StatsInfoKind},
    },
    server::server_manager::ServerData,
};
use strum::IntoEnumIterator;

use crate::components::{
    label::Label,
    separator::Separator,
    tabs::{TabContent, TabList, TabTrigger, Tabs, TabsVariant},
};

const LABEL_SIZE: i32 = 150;

#[component]
pub fn ChartsAtkUsedCount(atks_info: Vec<AtksInfo>) -> Element {
    let label_vector: Vec<String> = atks_info
        .iter()
        .map(|atk_info| format!("{} (count:{})", atk_info.atk_name, atk_info.nb_use))
        .collect();
    let nb_atk_used_acc: f32 = atks_info
        .iter()
        .map(|atk_info| atk_info.nb_use as f32)
        .sum();

    if nb_atk_used_acc == 0.0 {
        rsx! { "No attack used yet" }
    } else {
        rsx! {
            div { display: "grid", gap: "0.75rem", padding: "0 1rem",
                Label { html_for: "sheet-demo-name", "Total count: {nb_atk_used_acc}" }
                BarChart {
                    padding_top: 30,
                    padding_left: LABEL_SIZE,
                    padding_right: 50,
                    padding_bottom: 30,
                    bar_width: "5%",
                    horizontal_bars: true,
                    label_interpolation: (|v| format!("{v}%")) as fn(f32) -> String,
                    series: vec![
                        atks_info
                            .iter()
                            .map(|atk_info| (atk_info.nb_use as f32 / nb_atk_used_acc) * 100.0)
                            .collect(),
                    ],
                    labels: label_vector,
                    label_size: LABEL_SIZE,
                    class_bar_label: "bar-label",
                    class_grid_label: "bar-label",
                }
            }

        }
    }
}

#[component]
pub fn TotalAtksAmount(atks_info: Vec<AtksInfo>) -> Element {
    let total_real_heal = atks_info
        .iter()
        .map(|atk_info| {
            atk_info
                .totals_by_target
                .values()
                .map(|total| total.total_real_heal as f32)
                .sum::<f32>()
        })
        .sum::<f32>();
    let total_real_dmg = atks_info
        .iter()
        .map(|atk_info| {
            atk_info
                .totals_by_target
                .values()
                .map(|total| total.total_real_dmg.abs() as f32)
                .sum::<f32>()
        })
        .sum::<f32>();

    rsx! {
        div { display: "grid", gap: "0.75rem", padding: "0 1rem",
            Separator {
                style: "margin: 5px 0;",
                width: "80%",
                horizontal: true,
                decorative: true,
            }
            Label { html_for: "sheet-demo-name", "Total heal: {total_real_heal}" }
            Separator {
                style: "margin: 5px 0;",
                width: "80%",
                horizontal: true,
                decorative: true,
            }
            Label { html_for: "sheet-demo-name", "Total damage: {total_real_dmg}" }
            Separator {
                style: "margin: 5px 0;",
                width: "80%",
                horizontal: true,
                decorative: true,
            }
        }

    }
}

#[component]
pub fn TabStats() -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();

    // snap
    let server_data_snap = server_data();
    let game_state = &server_data_snap.core_game_data.game_manager.game_state;
    let heroes = server_data_snap
        .core_game_data
        .game_manager
        .pm
        .active_heroes
        .clone();

    // merged stats for "All" tab: accumulate atk_info from every hero
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

    // tab 0 = "All", then one tab per hero

    rsx! {
        Tabs {
            default_value: "tab0".to_string(),
            horizontal: true,
            max_width: "40em",
            TabList {
                TabTrigger { value: "tab0".to_string(), index: 0_usize, "All" }
                for (i, c) in heroes.clone().into_iter().enumerate() {
                    TabTrigger { value: format!("tab{}", i + 1), index: i + 1, "{c.id_name}" }
                }
            }

            // "All" tab content
            TabContent { value: "tab0".to_string(), index: 0_usize, width: "40em",
                div {
                    Tabs {
                        variant: TabsVariant::Default,
                        default_value: "tab1".to_string(),
                        horizontal: true,
                        max_width: "40em",
                        TabList {
                            for (j, s) in StatsInfoKind::iter().enumerate() {
                                TabTrigger { value: format!("tab{}", j + 1), index: j, "{s}" }
                            }
                        }
                        for (j, s) in StatsInfoKind::iter().enumerate() {
                            TabContent {
                                value: format!("tab{}", j + 1),
                                index: j,
                                width: "40em",
                                div {
                                    match s {
                                        StatsInfoKind::AtksCount => rsx! {
                                            ChartsAtkUsedCount { atks_info: all_atk_info.clone() }
                                        },
                                        StatsInfoKind::AtksAmount => rsx! {
                                            TotalAtksAmount { atks_info: all_atk_info.clone() }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Per-hero tabs
            for (i, c) in heroes.into_iter().enumerate() {
                TabContent {
                    value: format!("tab{}", i + 1),
                    index: i + 1,
                    width: "40em",
                    div {
                        Tabs {
                            variant: TabsVariant::Default,
                            default_value: "tab1".to_string(),
                            horizontal: true,
                            max_width: "40em",
                            TabList {
                                for (j, s) in StatsInfoKind::iter().enumerate() {
                                    TabTrigger {
                                        value: format!("tab{}", j + 1),
                                        index: j,
                                        "{s.to_string()}"
                                    }
                                }
                            }
                            for (j, s) in StatsInfoKind::iter().enumerate() {
                                TabContent {
                                    value: format!("tab{}", j + 1),
                                    index: j,
                                    width: "40em",
                                    div {
                                        match s {
                                            StatsInfoKind::AtksCount => rsx! {
                                                ChartsAtkUsedCount {
                                                    atks_info: game_state
                                                        .stats_in_game
                                                        .get(&c.id_name)
                                                        .unwrap_or(&StatsInGame::default())
                                                        .all_atk_info
                                                        .clone(),
                                                }
                                            },
                                            StatsInfoKind::AtksAmount => rsx! {
                                                TotalAtksAmount {
                                                    atks_info: game_state
                                                        .stats_in_game
                                                        .get(&c.id_name)
                                                        .unwrap_or(&StatsInGame::default())
                                                        .all_atk_info
                                                        .clone(),
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

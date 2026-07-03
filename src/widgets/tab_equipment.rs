use std::collections::BTreeMap;

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_i18n::t;
use dioxus_primitives::ContentSide;
use lib_rpg::{
    character_mod::{
        character::Character,
        equipment::{Equipment, EquipmentJsonKey},
        inventory::EquipmentInventory,
    },
    server::server_manager::ServerData,
};

use crate::{
    common::{CtxAppLang, SERVER_NAME, lang_from_app_lang},
    components::{
        button::{Button, ButtonVariant},
        tabs::{TabContent, TabList, TabTrigger, Tabs},
        tooltip::{Tooltip, TooltipContent, TooltipTrigger},
    },
    websocket_handler::event::{ClientEvent, ServerEvent},
};

#[component]
pub fn TabEquipment(c: Character) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let server_name = SERVER_NAME();
    let all_equipments_table = server_data()
        .core_game_data
        .game_manager
        .pm
        .equipment_table
        .clone();
    let inventory_equipments = c.inventory.get_all_equipments(
        all_equipments_table
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<Equipment>>()
            .as_slice(),
        false,
    );
    let flatten_inventory_equipments = inventory_equipments
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<Equipment>>();

    let ordered_equipments: BTreeMap<EquipmentJsonKey, Vec<EquipmentInventory>> =
        c.inventory.equipments.clone().into_iter().collect();

    // Track the active tab value as a reactive Signal so use_effect can re-run on change.
    let mut current_tab: Signal<String> = use_signal(|| "tab1".to_string());

    // Ordered list of category keys — used to map tab index back to category.
    let ordered_categories: Vec<EquipmentJsonKey> = ordered_equipments.keys().cloned().collect();
    let ordered_categories_eff = ordered_categories.clone();
    let char_id_eff = c.id_name.clone();

    // Every time the active tab changes (including on initial mount), send mark-seen
    // for that tab's category so the "new" badge disappears as soon as the user sees it.
    use_effect(move || {
        let tab = current_tab();
        if let Some(idx) = tab
            .strip_prefix("tab")
            .and_then(|n| n.parse::<usize>().ok())
            && let Some(category) = ordered_categories_eff.get(idx.saturating_sub(1))
        {
            let cat = category.to_string();
            let srv = server_name.clone();
            let cid = char_id_eff.clone();
            spawn(async move {
                let _ = socket
                    .send(ClientEvent::RequestMarkEquipSeen(cat, cid, srv))
                    .await;
            });
        }
    });

    rsx! {
        Tabs {
            default_value: "tab1".to_owned(),
            on_value_change: move |val: String| current_tab.set(val),
            horizontal: true,
            width: "100%",
            TabList {
                for (i, (key, items)) in ordered_equipments.iter().enumerate() {
                    {
                        let has_new = items.iter().any(|e| e.is_new);
                        let equipped_count = items.iter().filter(|e| e.is_equipped).count();
                        rsx! {
                            TabTrigger { value: format!("tab{}", i + 1), index: i,
                                span { class: "equip-tab-label",
                                    "{key}"
                                    if equipped_count > 0 {
                                        span {
                                            class: "equip-tab-equipped-badge",
                                            title: t!("equip-count-equipped", count : equipped_count as i64),
                                            "✓"
                                        }
                                    }
                                    if has_new {
                                        span {
                                            class: "equip-tab-new-badge",
                                            title: t!("equip-new-item"),
                                            "!"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for (i, (key, items)) in ordered_equipments.iter().enumerate() {
                {
                    let tab_key = key.clone();
                    rsx! {
                        TabContent { value: format!("tab{}", i + 1), index: i, width: "100%",
                            EquipmentTabContent {
                                category_key: tab_key,
                                items: items.clone(),
                                all_inventory_equipments: flatten_inventory_equipments.clone(),
                                character_id_name: c.id_name.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EquipmentTabContent(
    category_key: EquipmentJsonKey,
    items: Vec<EquipmentInventory>,
    all_inventory_equipments: Vec<Equipment>,
    character_id_name: String,
) -> Element {
    // Partition: equipped first, then unequipped
    let equipped: Vec<&EquipmentInventory> = items.iter().filter(|e| e.is_equipped).collect();
    let unequipped: Vec<&EquipmentInventory> = items.iter().filter(|e| !e.is_equipped).collect();

    rsx! {
        div { class: "equip-tab-content",
            if !equipped.is_empty() {
                div { class: "equip-section-title", {t!("equip-section-equipped")} }
            }
            for item in equipped {
                EquipmentTooltip {
                    e_inventory: item.clone(),
                    all_inventory_equipments: all_inventory_equipments.clone(),
                    character_id_name: character_id_name.clone(),
                }
            }
            if !unequipped.is_empty() {
                div { class: "equip-section-title", {t!("equip-section-in-bag")} }
            }
            for item in unequipped {
                EquipmentTooltip {
                    e_inventory: item.clone(),
                    all_inventory_equipments: all_inventory_equipments.clone(),
                    character_id_name: character_id_name.clone(),
                }
            }
            if items.is_empty() {
                div { class: "equip-empty", {t!("equip-empty-slot")} }
            }
        }
    }
}

#[component]
fn EquipmentTooltip(
    e_inventory: EquipmentInventory,
    all_inventory_equipments: Vec<Equipment>,
    character_id_name: String,
) -> Element {
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let app_lang = use_context::<CtxAppLang>().0;

    let e_inventory_name = e_inventory.unique_name.clone();
    let is_new = e_inventory.is_new;
    let lang = lang_from_app_lang(&app_lang());

    let equipment = match all_inventory_equipments
        .iter()
        .find(|e| e.unique_name == e_inventory.unique_name)
    {
        Some(equipment) => equipment,
        None => {
            return rsx! {
                div { {t!("equip-not-found")} }
            };
        }
    };
    let stats = equipment
        .stats
        .all_stats
        .iter()
        .filter_map(|(stat_name, stat_value)| {
            if stat_value.buf_equip_value == 0 && stat_value.buf_equip_percent == 0 {
                None
            } else {
                Some(format!("{}: +{}", stat_name, stat_value.buf_equip_value))
            }
        })
        .collect::<Vec<String>>();

    rsx! {
        div { class: "equip-item-row",
            Tooltip {
                TooltipTrigger {
                    div {
                        Button {
                            variant: if e_inventory.is_equipped { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                            width: "15em",
                            onclick: move |_| {
                                let mv_e_inventory_name = e_inventory_name.clone();
                                let mv_character_id_name = character_id_name.clone();
                                async move {
                                    let _ = socket
                                        .send(
                                            ClientEvent::RequestToggleEquip(
                                                mv_e_inventory_name,
                                                mv_character_id_name,
                                                SERVER_NAME(),
                                            ),
                                        )
                                        .await;
                                }
                            },
                            span { class: "equip-btn-label",
                                if is_new {
                                    span {
                                        class: "equip-new-dot",
                                        title: t!("equip-new-dot-title"),
                                        "🆕 "
                                    }
                                }
                                "{equipment.name_for(lang)}"
                                if e_inventory.is_equipped {
                                    span { class: "equip-equipped-check", " ✓" }
                                }
                            }
                        }
                    }
                }
                TooltipContent { side: ContentSide::Right,
                    p { style: "margin:0 0 4px 0; font-weight:600; color:var(--rpg-gold,#c9a227);",
                        "{equipment.name_for(lang)}"
                    }
                    if stats.is_empty() {
                        p { style: "margin:0; color: var(--rpg-text-muted,#8a8fa8); font-style:italic;",
                            {t!("equip-no-stat-bonuses")}
                        }
                    } else {
                        for stat in stats {
                            p { style: "margin: 0;", "{stat}" }
                        }
                    }
                    p { style: "margin:4px 0 0 0; font-size:0.72rem; color:var(--rpg-text-muted,#8a8fa8);",
                        if e_inventory.is_equipped {
                            {t!("equip-click-to-unequip")}
                        } else {
                            {t!("equip-click-to-equip")}
                        }
                    }
                }
            }
        }
    }
}

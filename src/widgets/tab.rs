use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use dioxus::prelude::*;
use dioxus_primitives::{ContentSide, scroll_area::ScrollDirection};
use lib_rpg::{
    character_mod::{
        character::Character,
        equipment::{Equipment, EquipmentJsonKey},
        inventory::EquipmentInventory,
    },
    server::server_manager::ServerData,
};

use crate::components::{
    button::{Button, ButtonVariant},
    label::Label,
    scroll_area::ScrollArea,
    tabs::{TabContent, TabList, TabTrigger, Tabs},
    tooltip::{Tooltip, TooltipContent, TooltipTrigger},
};

#[component]
pub fn TabEquipment(c: Character) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
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

    let ordered_equipments: BTreeMap<String, Vec<EquipmentInventory>> =
        c.inventory.equipments.clone().into_iter().collect();
    rsx! {
        Tabs {
            default_value: "tab1".to_string(),
            horizontal: true,
            max_width: "16em",
            TabList {
                for (i , e) in ordered_equipments.iter().enumerate() {
                    TabTrigger { value: format!("tab{}", i + 1), index: i, "{e.0}" }
                }
            }
            for (i , e) in ordered_equipments.iter().enumerate() {
                TabContent { value: format!("tab{}", i + 1), index: i, width: "17em",
                    div {
                        for (j , item) in e.1.iter().enumerate() {
                            EquipmentTooltip {
                                e_inventory: item.clone(),
                                all_inventory_equipments: flatten_inventory_equipments.clone(),
                            }
                        }
                    }

                }
            }
        }
    }
}

#[component]
fn EquipmentTooltip(
    e_inventory: EquipmentInventory,
    all_inventory_equipments: Vec<Equipment>,
) -> Element {
    let equipment = match all_inventory_equipments
        .iter()
        .find(|e| e.unique_name == e_inventory.unique_name)
    {
        Some(equipment) => equipment,
        None => {
            return rsx! {
                div { "Equipment not found" }
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
                Some(format!("{}: {}", stat_name, stat_value.buf_equip_value))
            }
        })
        .collect::<Vec<String>>();
    rsx! {
        div { class: "character-effects",
            Tooltip {
                TooltipTrigger {
                    div {
                        Button {
                            variant: if e_inventory.is_equipped { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                            width: "16em",
                            onclick: move |_| async move {} // update stats display,
                            "{e_inventory.unique_name}"
                        }
                    }
                }
                TooltipContent { side: ContentSide::Right,
                    for stat in stats {
                        p { style: "margin: 0;", "{stat}" }
                    }
                }
            }
        }

    }
}

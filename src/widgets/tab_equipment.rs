use std::collections::BTreeMap;

use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
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
    common::SERVER_NAME,
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
    rsx! {
        Tabs {
            default_value: "tab1".to_string(),
            horizontal: true,
            max_width: "17em",
            TabList {
                for (i, e) in ordered_equipments.iter().enumerate() {
                    TabTrigger { value: format!("tab{}", i + 1), index: i, "{e.0}" }
                }
            }
            for (i, e) in ordered_equipments.iter().enumerate() {
                TabContent { value: format!("tab{}", i + 1), index: i, width: "17em",
                    div {
                        for (_j, item) in e.1.iter().enumerate() {
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
    // context
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let local_login_name_session = use_context::<Signal<String>>();

    let e_inventory_name = e_inventory.unique_name.clone();

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
                            width: "17em",
                            onclick: move |_| {
                                let mv_e_inventory_name = e_inventory_name.clone();
                                async move {
                                    // send equip/unequip request
                                    let _ = socket
                                        .send(
                                            ClientEvent::RequestToggleEquip(
                                                mv_e_inventory_name,
                                                local_login_name_session(),
                                                SERVER_NAME(),
                                            ),
                                        )
                                        .await;
                                }
                            },
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

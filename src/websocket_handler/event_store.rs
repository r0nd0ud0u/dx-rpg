#[cfg(feature = "server")]
use crate::common::DATA_MANAGER;
#[cfg(feature = "server")]
use crate::websocket_handler::common_event::SERVER_MANAGER;
#[cfg(feature = "server")]
use dioxus::logger::tracing;
#[cfg(feature = "server")]
use lib_rpg::common::log_data::LogData;
#[cfg(feature = "server")]
use lib_rpg::shop::{build_consumable_by_name, sell_price};
#[cfg(feature = "server")]
use lib_rpg::utils;

/// Buy an item (equipment or consumable) for a character.
/// Deducts the catalog price from the character's money and adds the item to the bag.
#[cfg(feature = "server")]
pub fn buy_item_handler(
    server_name: &str,
    character_id_name: &str,
    item_name: &str,
    item_kind: &str,
) {
    let price = {
        let dm = DATA_MANAGER.lock().unwrap();
        dm.shop_catalog
            .iter()
            .find(|i| i.name == item_name)
            .map(|i| i.price)
            .unwrap_or(0)
    };

    if price == 0 {
        tracing::warn!(
            "buy_item_handler: item '{}' not found in shop catalog",
            item_name
        );
        return;
    }

    let mut sm = SERVER_MANAGER.lock().unwrap();
    let Some(server_data) = sm.servers_data.get_mut(server_name) else {
        tracing::error!("buy_item_handler: no server data for '{}'", server_name);
        return;
    };
    let pm = &mut server_data.core_game_data.game_manager.pm;
    let Some(hero) = pm
        .active_heroes
        .iter_mut()
        .find(|h| h.id_name == character_id_name)
    else {
        tracing::error!(
            "buy_item_handler: character '{}' not found",
            character_id_name
        );
        return;
    };

    let mut purchase_log: Option<LogData> = None;

    if item_kind == "Consumable" {
        let Some(consumable) = build_consumable_by_name(item_name) else {
            tracing::error!("buy_item_handler: unknown consumable '{}'", item_name);
            return;
        };
        match hero.inventory.buy_consumable(consumable, price) {
            Ok(()) => {
                tracing::info!(
                    "{} bought consumable '{}' for {} gold",
                    character_id_name,
                    item_name,
                    price
                );
                purchase_log = Some(LogData {
                    message: utils::format_string_with_timestamp(&format!(
                        "🛒 {} bought {} for {} gold",
                        character_id_name, item_name, price
                    )),
                    color: String::new(),
                });
            }
            Err(e) => tracing::warn!("buy_item_handler consumable: {}", e),
        }
    } else if item_kind == "Equipment" {
        let dm = DATA_MANAGER.lock().unwrap();
        let equipment = dm
            .equipment_table
            .values()
            .flatten()
            .find(|e| e.unique_name == item_name)
            .cloned();
        drop(dm);

        let Some(equip) = equipment else {
            tracing::error!("buy_item_handler: equipment '{}' not found", item_name);
            return;
        };
        match hero.inventory.buy_equipment(&equip, price) {
            Ok(()) => {
                tracing::info!(
                    "{} bought equipment '{}' for {} gold",
                    character_id_name,
                    item_name,
                    price
                );
                purchase_log = Some(LogData {
                    message: utils::format_string_with_timestamp(&format!(
                        "🛒 {} bought {} for {} gold",
                        character_id_name, item_name, price
                    )),
                    color: String::new(),
                });
            }
            Err(e) => tracing::warn!("buy_item_handler equipment: {}", e),
        }
    }

    // Do NOT call pm.modify_active_character here — that copies current_player
    // (the active combat player) back over the hero, erasing the purchase.
    // We modified the hero directly via active_heroes.iter_mut(), which is enough.

    if let Some(entry) = purchase_log {
        server_data.core_game_data.game_manager.logs.push(entry);
    }
}

/// Sell an item (equipment or consumable) from a character's bag.
/// Adds 50 % of the catalog price back to the character's money.
#[cfg(feature = "server")]
pub fn sell_item_handler(
    server_name: &str,
    character_id_name: &str,
    item_name: &str,
    item_kind: &str,
) {
    let buy_price = {
        let dm = DATA_MANAGER.lock().unwrap();
        dm.shop_catalog
            .iter()
            .find(|i| i.name == item_name)
            .map(|i| i.price)
            .unwrap_or(0)
    };
    let refund = sell_price(buy_price);

    let mut sm = SERVER_MANAGER.lock().unwrap();
    let Some(server_data) = sm.servers_data.get_mut(server_name) else {
        tracing::error!("sell_item_handler: no server data for '{}'", server_name);
        return;
    };
    let pm = &mut server_data.core_game_data.game_manager.pm;
    let Some(hero) = pm
        .active_heroes
        .iter_mut()
        .find(|h| h.id_name == character_id_name)
    else {
        tracing::error!(
            "sell_item_handler: character '{}' not found",
            character_id_name
        );
        return;
    };

    let mut sale_log: Option<LogData> = None;

    if item_kind == "Consumable" {
        match hero.inventory.sell_consumable(item_name, refund) {
            Ok(()) => {
                tracing::info!(
                    "{} sold consumable '{}' for {} gold",
                    character_id_name,
                    item_name,
                    refund
                );
                sale_log = Some(LogData {
                    message: utils::format_string_with_timestamp(&format!(
                        "💰 {} sold {} for {} gold",
                        character_id_name, item_name, refund
                    )),
                    color: String::new(),
                });
            }
            Err(e) => tracing::warn!("sell_item_handler consumable: {}", e),
        }
    } else if item_kind == "Equipment" {
        match hero.inventory.sell_equipment(item_name, refund) {
            Ok(()) => {
                tracing::info!(
                    "{} sold equipment '{}' for {} gold",
                    character_id_name,
                    item_name,
                    refund
                );
                sale_log = Some(LogData {
                    message: utils::format_string_with_timestamp(&format!(
                        "💰 {} sold {} for {} gold",
                        character_id_name, item_name, refund
                    )),
                    color: String::new(),
                });
            }
            Err(e) => tracing::warn!("sell_item_handler equipment: {}", e),
        }
    }

    if let Some(entry) = sale_log {
        server_data.core_game_data.game_manager.logs.push(entry);
    }
}

use dioxus::prelude::*;
use dioxus::logger::tracing;
use lib_rpg::character::Character;

use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};

#[component]
pub fn TabDemo(c: Character) -> Element {
    // collect in a hashmap the equipment on with the index as key and the name of the item as value
    let map: std::collections::HashMap<&String, String> = c.equipment_on.iter().map(|(i, e)| (i, e.first().map(|item| item.name.clone()).unwrap_or_else(|| "Empty".to_string()))).collect::<std::collections::HashMap<_, _>>();

    rsx! {
        Tabs {
            default_value: "tab1".to_string(),
            horizontal: true,
            max_width: "16rem",
            TabList {
                for (i, e) in c.equipment_on.iter().enumerate() {
                    TabTrigger { value: format!("tab{}", i + 1), index: i, "{e.0}" }
                }
            }
            for (i, e) in c.equipment_on.iter().enumerate() {
                TabContent { value: format!("tab{}", i + 1), index: i, "{map.get(&e.0).unwrap_or(&\"Empty\".to_string())}" }
            }
        }
    }
}

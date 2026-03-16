use dioxus::prelude::*;
use dioxus_primitives::scroll_area::ScrollDirection;
use lib_rpg::character_mod::character::Character;

use crate::components::{
    label::Label,
    scroll_area::ScrollArea,
    tabs::{TabContent, TabList, TabTrigger, Tabs},
};

#[component]
pub fn TabDemo(c: Character) -> Element {
    // collect in a hashmap the equipment on with the index as key and the name of the item as value
    let map: std::collections::HashMap<&String, String> = c
        .equipment_on
        .iter()
        .map(|(i, e)| {
            (
                i,
                e.first()
                    .map(|item| item.name.clone())
                    .unwrap_or_else(|| "Empty".to_string()),
            )
        })
        .collect::<std::collections::HashMap<_, _>>();

    rsx! {
        Tabs {
            default_value: "tab1".to_string(),
            horizontal: true,
            max_width: "16rem",
            TabList {
                for (i , e) in c.equipment_on.iter().enumerate() {
                    TabTrigger { value: format!("tab{}", i + 1), index: i, "{e.0}" }
                }
            }
            for (i , e) in c.equipment_on.iter().enumerate() {
                TabContent { value: format!("tab{}", i + 1), index: i,
                    ScrollArea {
                        //width: "100%",
                        height: "30em",
                        border: "1px solid var(--primary-color-6)",
                        border_radius: "0.5em",
                        padding: "0 1em 1em 1em",
                        direction: ScrollDirection::Vertical,
                        tabindex: "0",
                        div { class: "scroll-content",
                            Label { html_for: "sheet-demo-name",
                                "Equipped item: {map.get(&e.0).unwrap_or(&\"Empty\".to_string())}"
                            }
                        }
                    }

                }
            }
        }
    }
}

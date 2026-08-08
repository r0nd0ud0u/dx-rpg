use dioxus::prelude::*;

pub use super::primitive::use_drag_and_drop_list_order;
use super::primitive::{self, DragAndDropListProps};

// Loaded once from the app root (main.rs) instead of via a nested document::Link here —
// dioxus-desktop doesn't inject document::Link stylesheets declared inside a child
// component's own render into <head>, only ones declared at the App() root.
pub const STYLE_CSS: Asset = asset!("./style.css");

/// A reorderable list of `items` (rendered labels), each identified by the
/// matching entry in `item_keys`. Read the final order back out (e.g. on a
/// "Save" click) with [`use_drag_and_drop_list_order`] from within `children`.
#[component]
pub fn DragAndDropList(props: DragAndDropListProps) -> Element {
    rsx! {
        primitive::DragAndDropList {
            items: props.items,
            item_keys: props.item_keys,
            aria_label: props.aria_label,
            attributes: props.attributes,
            {props.children}
        }
    }
}

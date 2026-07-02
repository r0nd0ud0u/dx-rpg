use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::{
    auth_manager::server_fn::is_admin_enabled,
    board_game_components::{
        admin_tab_characters::AdminCharactersTab, admin_tab_equipment::AdminEquipmentTab,
        admin_tab_scenarios::AdminScenariosTab, admin_tab_users::AdminUsersTab,
    },
};

#[derive(Clone, PartialEq)]
enum AdminTab {
    Users,
    Scenarios,
    Characters,
    Equipment,
}

#[component]
pub fn AdminPage() -> Element {
    let mut admin_enabled = use_signal(|| true);
    let mut tab = use_signal(|| AdminTab::Users);

    use_effect(move || {
        spawn(async move {
            if let Ok(enabled) = is_admin_enabled().await {
                admin_enabled.set(enabled);
            }
        });
    });

    if !admin_enabled() {
        return rsx! {
            div { class: "home-container",
                h2 { class: "rpg-title", {t!("admin-panel-title")} }
                p { class: "rpg-subtitle", {t!("admin-panel-disabled")} }
            }
        };
    }

    rsx! {
        div { class: "admin-page-container",
            h2 { class: "rpg-title", {t!("admin-panel-title")} }

            div { class: "admin-tabs",
                button {
                    class: if tab() == AdminTab::Users { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Users),
                    {t!("admin-tab-users")}
                }
                button {
                    class: if tab() == AdminTab::Scenarios { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Scenarios),
                    {t!("admin-tab-scenarios")}
                }
                button {
                    class: if tab() == AdminTab::Characters { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Characters),
                    {t!("admin-tab-characters")}
                }
                button {
                    class: if tab() == AdminTab::Equipment { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Equipment),
                    {t!("admin-tab-equipment")}
                }
            }

            match tab() {
                AdminTab::Users => rsx! {
                    AdminUsersTab {}
                },
                AdminTab::Scenarios => rsx! {
                    AdminScenariosTab {}
                },
                AdminTab::Characters => rsx! {
                    AdminCharactersTab {}
                },
                AdminTab::Equipment => rsx! {
                    AdminEquipmentTab {}
                },
            }
        }
    }
}

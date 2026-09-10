use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::{
    auth_manager::server_fn::{get_permissions, is_admin_enabled, logout},
    board_game_components::{
        admin_tab_characters::AdminCharactersTab, admin_tab_equipment::AdminEquipmentTab,
        admin_tab_scenarios::AdminScenariosTab, admin_tab_users::AdminUsersTab,
        common_comp::BackButton,
    },
    common::{ADMIN, CtxSessionExpired, DISCONNECTED_USER, Route},
    game_channel::GameChannel,
    websocket_handler::{NO_CLIENT_ID, event::ClientEvent},
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
    let socket = use_context::<GameChannel>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    let mut session_expired = use_context::<CtxSessionExpired>().0;
    let navigator = use_navigator();

    let mut admin_enabled = use_signal(|| true);
    let mut session_ok = use_signal(|| true);
    let mut tab = use_signal(|| AdminTab::Users);

    use_effect(move || {
        spawn(async move {
            if let Ok(enabled) = is_admin_enabled().await {
                admin_enabled.set(enabled);
            }
        });
    });

    // `require_admin` protects the endpoints themselves; checking the live session here too
    // turns tab-by-tab silent failures into one clear message, and tells "session expired"
    // apart from "never was Admin".
    use_effect(move || {
        spawn(async move {
            let is_admin =
                matches!(get_permissions().await, Ok(perms) if perms.contains("Admin::View"));
            session_ok.set(is_admin);
            if !is_admin && local_login_name_session() == *ADMIN {
                // Session expired or revoked; local storage never clears itself. Drop the
                // stale identity and let LoginPage explain. A non-admin user instead gets
                // "access denied" below, session untouched.
                let _ = logout().await;
                let _ = socket
                    .send(ClientEvent::RequestLogOut(local_login_name_session()))
                    .await;
                *local_login_name_session.write() = DISCONNECTED_USER.clone();
                *local_login_id_session.write() = NO_CLIENT_ID;
                session_expired.set(true);
                navigator.push(Route::Home {});
            }
        });
    });

    if !admin_enabled() {
        return rsx! {
            div { class: "home-container",
                BackButton { target: Route::Home {}.into() }
                h2 { class: "rpg-title", {t!("admin-panel-title")} }
                p { class: "rpg-subtitle", {t!("admin-panel-disabled")} }
            }
        };
    }

    if !session_ok() {
        return rsx! {
            div { class: "home-container",
                BackButton { target: Route::Home {}.into() }
                h2 { class: "rpg-title", {t!("admin-panel-title")} }
                p { class: "rpg-subtitle", {t!("admin-panel-access-denied")} }
            }
        };
    }

    rsx! {
        div { class: "admin-page-container",
            BackButton { target: Route::Home {}.into() }
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

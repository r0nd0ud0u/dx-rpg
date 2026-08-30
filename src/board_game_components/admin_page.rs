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

    // Server-side re-check of the *live* session, not just the client's persisted
    // username: `require_admin` (server_fn/auth.rs) is what actually protects every
    // admin endpoint now, but discovering that tab-by-tab (each panel's own list/save
    // call quietly failing) would surface as a handful of silently-logged errors instead
    // of one clear message. Checking here, before any tab renders, catches it in one
    // place — and distinguishes "this session expired" from "this user was never Admin"
    // (see below), rather than treating both the same way.
    use_effect(move || {
        spawn(async move {
            let is_admin =
                matches!(get_permissions().await, Ok(perms) if perms.contains("Admin::View"));
            session_ok.set(is_admin);
            if !is_admin && local_login_name_session() == *ADMIN {
                // The client still believes it's signed in as Admin, but the server
                // disagrees — the session expired (or was revoked) since local storage
                // never clears on its own. Drop the stale local identity so the rest of
                // the app stops acting as if it's still authenticated, and let LoginPage
                // explain why. A regular, non-admin user landing here instead just gets
                // "access denied" below, with their own session left untouched.
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

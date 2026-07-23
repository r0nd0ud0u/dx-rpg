use crate::{
    common::DISCONNECTED_USER,
    websocket_handler::{NO_CLIENT_ID, msg_from_client::send_disconnect_from_server_data},
};
use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use dioxus_i18n::t;
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::{
    auth_manager::server_fn::logout,
    common::{ADMIN, CtxAppLang, CtxSyncedInsecureCerts, CtxSyncedServerUrl, Route},
    components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
            AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
        },
        button::{Button, ButtonVariant},
        input::Input,
        sidebar::{Sidebar, SidebarTrigger},
    },
    websocket_handler::{
        event::{ClientEvent, ServerEvent},
        msg_from_client::send_disconnect_from_server_data as send_quit,
    },
};

/// Whether the admin-panel link should be shown for this logged-in username.
fn is_admin_link_visible(username: &str) -> bool {
    username == ADMIN.as_str()
}

/// Whether the "Quit game" trigger should be shown for this game phase.
fn is_quit_visible(phase: &GamePhase) -> bool {
    *phase == GamePhase::Running
}

/// Whether the current username represents a signed-in user (vs. the
/// disconnected placeholder), i.e. whether the sign-out label/state applies.
fn is_signed_in(username: &str) -> bool {
    username != DISCONNECTED_USER.as_str()
}

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    let server_data = use_context::<Signal<ServerData>>();
    let mut app_lang = use_context::<CtxAppLang>().0;
    // Native clients only — see the doc comment on CtxSyncedServerUrl in common.rs for why
    // these are declared in App() and consumed here via context rather than called
    // directly with use_synced_storage in Navbar (a #[layout(...)] component).
    let mut synced_server_url = use_context::<CtxSyncedServerUrl>().0;
    let mut synced_insecure_certs = use_context::<CtxSyncedInsecureCerts>().0;

    // nav
    let navigator = use_navigator();

    // dialog open states — lifted here so the roots can live outside the navbar div
    let mut help_open = use_signal(|| false);
    let mut quit_open = use_signal(|| false);

    // Server connection settings dialog — native only (gated at render time below via
    // `cfg!(target_arch = "wasm32")`, since #[cfg] attributes aren't supported inside
    // rsx!;).
    let mut server_settings_open = use_signal(|| false);
    // Mobile-only nav drawer (Sidebar) — the desktop controls group is duplicated
    // into it (CSS-gated visibility, see .navbar-desktop-group/.navbar-mobile-trigger
    // in main.css) so narrow screens get a proper drawer instead of a cramped row.
    let mut mobile_nav_open = use_signal(|| false);
    // Draft state so typing doesn't write to storage on every keystroke; populated from
    // the synced values each time the dialog is opened (see the trigger button below).
    let mut server_url_draft = use_signal(String::new);
    let mut insecure_certs_draft = use_signal(|| false);
    let mut server_saved_message = use_signal(|| false);

    // snapshot
    let snap_local_login_name_session = local_login_name_session();

    rsx! {
        div { class: "page-layout",
            // ── Navbar bar ────────────────────────────────────────────────────────
            div { class: "navbar",
                // Left: brand + admin panel link
                div { style: "display: flex; align-items: center; gap: 1rem;",
                    Link {
                        class: "navbar-brand",
                        to: Route::Home {},
                        onclick: move |_| async move {
                            send_disconnect_from_server_data(socket, &local_login_name_session()).await;
                        },
                        "⚔️ RPG"
                    }
                    if is_admin_link_visible(&snap_local_login_name_session) {
                        Link {
                            class: "navbar-admin-link",
                            to: Route::AdminPage {},
                            {t!("navbar-admin-panel")}
                        }
                    }
                }
                // Mobile-only hamburger trigger — opens the Sidebar drawer below.
                // Hidden on desktop, shown ≤768px (see .navbar-mobile-trigger in main.css).
                div { class: "navbar-mobile-trigger",
                    SidebarTrigger { open: mobile_nav_open, label: t!("navbar-menu-open") }
                }
                // Right: trigger buttons only (no dialog roots here). Hidden on mobile —
                // see .navbar-desktop-group in main.css — and duplicated into the Sidebar
                // drawer below for narrow screens.
                div { class: "navbar-desktop-group",
                    // Language dropdown — current language is the selected option.
                    select {
                        class: "navbar-lang-select",
                        "aria-label": t!("lang-select-label"),
                        value: "{app_lang()}",
                        onchange: move |e| app_lang.set(e.value()),
                        option { value: "en", "🇬🇧 English" }
                        option { value: "fr", "🇫🇷 Français" }
                    }
                    // Help trigger
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| help_open.set(true),
                        "?"
                    }
                    // Server settings trigger (native only — excluded from web-server SSR
                    // so the hydration stream matches the wasm32 client's render)
                    if cfg!(all(not(target_arch = "wasm32"), not(feature = "server"))) {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                server_url_draft.set(synced_server_url());
                                insecure_certs_draft.set(synced_insecure_certs());
                                server_saved_message.set(false);
                                server_settings_open.set(true);
                            },
                            {t!("navbar-server-settings")}
                        }
                    }
                    // Quit-game trigger (only while a game is running)
                    if is_quit_visible(&server_data().core_game_data.game_phase) {
                        Button {
                            style: "width: 190px; box-sizing: border-box; text-align: center; white-space: nowrap;",
                            onclick: move |_| quit_open.set(true),
                            r#type: "button",
                            {t!("navbar-quit-game")}
                        }
                    }
                    if is_signed_in(&snap_local_login_name_session) {
                        span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                    }
                    Button {
                        style: "width: 160px; box-sizing: border-box; text-align: center; white-space: nowrap;",
                        variant: if is_signed_in(&snap_local_login_name_session) { ButtonVariant::Destructive } else { ButtonVariant::Secondary },
                        onclick: move |_| async move {
                            if local_login_name_session() != *DISCONNECTED_USER {
                                match logout().await {
                                    Ok(_) => {
                                        tracing::info!("{} is logged out", local_login_name_session());
                                        let _ = socket
                                            .clone()
                                            .send(ClientEvent::RequestLogOut(local_login_name_session()))
                                            .await;
                                        *local_login_name_session.write() = (*DISCONNECTED_USER).to_string();
                                        *local_login_id_session.write() = NO_CLIENT_ID;
                                    }
                                    Err(_) => {
                                        tracing::info!("Error on {} logout", local_login_name_session())
                                    }
                                }
                            }
                            navigator.push(Route::Home {});
                        },
                        if is_signed_in(&snap_local_login_name_session) {
                            {t!("navbar-sign-out")}
                        } else {
                            {t!("navbar-sign-in")}
                        }
                    }
                }
            }

            // ── Dialog roots — rendered at layout level, NOT inside the navbar div ──

            // Help dialog
            AlertDialogRoot { open: help_open(), on_open_change: move |v| help_open.set(v),
                AlertDialogContent {
                    AlertDialogTitle { {t!("help-title")} }
                    AlertDialogDescription {
                        div { style: "text-align:left; line-height:1.8; max-height:70vh; overflow-y:auto; padding-right:4px;",
                            // Getting started
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-bottom:2px;",
                                {t!("help-section-getting-started")}
                            }
                            p { {t!("help-step-1")} }
                            p { {t!("help-step-2")} }
                            p { {t!("help-step-3")} }

                            // Game mode
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-game-modes")}
                            }
                            p { {t!("help-mode-multiplayer")} }
                            p { {t!("help-mode-singleplayer")} }

                            // Lobby & character selection
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-lobby")}
                            }
                            p { {t!("help-step-4")} }
                            p { {t!("help-step-5")} }

                            // Combat
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-combat")}
                            }
                            p { {t!("help-step-6")} }
                            p { {t!("help-step-7")} }
                            p { {t!("help-step-8")} }

                            // Toolbar
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-toolbar")}
                            }
                            p { {t!("help-step-9")} }
                            p { {t!("help-step-10")} }
                            p { {t!("help-step-11")} }
                            p { {t!("help-step-12")} }

                            // Overworld
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-overworld")}
                            }
                            p { {t!("help-step-13")} }
                            p {
                                "    "
                                {t!("help-overworld-move")}
                            }
                            p {
                                "    "
                                {t!("help-overworld-interact")}
                            }
                            p {
                                "    "
                                {t!("help-overworld-encounter")}
                            }
                            p {
                                "    "
                                {t!("help-overworld-boss")}
                            }
                            p {
                                "    "
                                {t!("help-overworld-unlock")}
                            }
                            p {
                                "    "
                                {t!("help-overworld-back")}
                            }

                            // Store
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-store")}
                            }
                            p { {t!("help-step-14")} }
                            p {
                                "    "
                                {t!("help-store-equipment")}
                            }
                            p {
                                "    "
                                {t!("help-store-consumables")}
                            }
                            p {
                                "    "
                                {t!("help-store-bag")}
                            }
                            p {
                                "    "
                                {t!("help-store-gold")}
                            }

                            // Progression
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-progression")}
                            }
                            p { {t!("help-step-15")} }
                            p { {t!("help-step-16")} }
                            p { {t!("help-step-17")} }
                            p { {t!("help-step-18")} }

                            // Admin
                            p { style: "font-weight:700; color:var(--rpg-gold); margin-top:8px; margin-bottom:2px;",
                                {t!("help-section-admin")}
                            }
                            p { {t!("help-step-19")} }
                            p {
                                "    "
                                {t!("help-admin-users")}
                            }
                            p {
                                "    "
                                {t!("help-admin-characters")}
                            }
                            p {
                                "    "
                                {t!("help-admin-scenarios")}
                            }
                        }
                    }
                    AlertDialogAction {
                        AlertDialogCancel { {t!("common-close")} }
                    }
                }
            }

            // Server settings dialog (native only — excluded from web-server SSR)
            if cfg!(all(not(target_arch = "wasm32"), not(feature = "server"))) {
                AlertDialogRoot {
                    open: server_settings_open(),
                    on_open_change: move |v| server_settings_open.set(v),
                    AlertDialogContent {
                        AlertDialogTitle { {t!("server-settings-title")} }
                        AlertDialogDescription {
                            div { style: "text-align:left; display:flex; flex-direction:column; gap:0.75rem;",
                                p {
                                    {
                                        t!(
                                            "server-settings-current", url : dioxus::fullstack::get_server_url()
                                            .to_owned()
                                        )
                                    }
                                }
                                Input {
                                    placeholder: t!("server-settings-placeholder"),
                                    r#type: "text",
                                    value: "{server_url_draft}",
                                    oninput: move |e: FormEvent| server_url_draft.set(e.value()),
                                }
                                label { style: "display:flex; align-items:center; gap:0.5rem; cursor:pointer;",
                                    input {
                                        r#type: "checkbox",
                                        checked: insecure_certs_draft(),
                                        onchange: move |e: FormEvent| insecure_certs_draft.set(e.checked()),
                                    }
                                    span { {t!("server-settings-insecure-label")} }
                                }
                                p { style: "font-size:0.85em; color:var(--rpg-text-muted);",
                                    {t!("server-settings-insecure-warning")}
                                }
                                if server_saved_message() {
                                    p { style: "color:var(--rpg-gold); font-weight:600;",
                                        {t!("server-settings-saved")}
                                    }
                                }
                            }
                        }
                        AlertDialogActions {
                            AlertDialogCancel { {t!("common-cancel")} }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {
                                    synced_server_url.set(server_url_draft());
                                    synced_insecure_certs.set(insecure_certs_draft());
                                    server_saved_message.set(true);
                                },
                                {t!("server-settings-save")}
                            }
                        }
                    }
                }
            }

            // Quit-game confirmation dialog
            AlertDialogRoot { open: quit_open(), on_open_change: move |v| quit_open.set(v),
                AlertDialogContent {
                    AlertDialogTitle { {t!("quit-dialog-title")} }
                    AlertDialogDescription { {t!("quit-dialog-body")} }
                    AlertDialogAction {
                        AlertDialogCancel { {t!("common-cancel")} }
                        AlertDialogAction {
                            on_click: move |_| {
                                async move {
                                    send_quit(socket, &local_login_name_session()).await;
                                    let navigator = use_navigator();
                                    navigator.push(Route::Home {});
                                }
                            },
                            {t!("common-confirm")}
                        }
                    }
                }
            }

            // ── Mobile navigation drawer ─────────────────────────────────────────
            // Duplicates the desktop controls group above for narrow screens (see
            // .navbar-desktop-group / .navbar-mobile-trigger in main.css) — every
            // action also closes the drawer after firing.
            Sidebar { open: mobile_nav_open, title: Some(t!("navbar-menu-title")),
                select {
                    class: "navbar-lang-select",
                    "aria-label": t!("lang-select-label"),
                    value: "{app_lang()}",
                    onchange: move |e| app_lang.set(e.value()),
                    option { value: "en", "🇬🇧 English" }
                    option { value: "fr", "🇫🇷 Français" }
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        help_open.set(true);
                        mobile_nav_open.set(false);
                    },
                    {t!("help-title")}
                }
                if cfg!(all(not(target_arch = "wasm32"), not(feature = "server"))) {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            server_url_draft.set(synced_server_url());
                            insecure_certs_draft.set(synced_insecure_certs());
                            server_saved_message.set(false);
                            server_settings_open.set(true);
                            mobile_nav_open.set(false);
                        },
                        {t!("navbar-server-settings")}
                    }
                }
                if is_quit_visible(&server_data().core_game_data.game_phase) {
                    Button {
                        onclick: move |_| {
                            quit_open.set(true);
                            mobile_nav_open.set(false);
                        },
                        r#type: "button",
                        {t!("navbar-quit-game")}
                    }
                }
                if is_signed_in(&snap_local_login_name_session) {
                    span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                }
                Button {
                    variant: if is_signed_in(&snap_local_login_name_session) { ButtonVariant::Destructive } else { ButtonVariant::Secondary },
                    onclick: move |_| async move {
                        if local_login_name_session() != *DISCONNECTED_USER {
                            match logout().await {
                                Ok(_) => {
                                    tracing::info!("{} is logged out", local_login_name_session());
                                    let _ = socket
                                        .clone()
                                        .send(ClientEvent::RequestLogOut(local_login_name_session()))
                                        .await;
                                    *local_login_name_session.write() = (*DISCONNECTED_USER).to_string();
                                    *local_login_id_session.write() = NO_CLIENT_ID;
                                }
                                Err(_) => {
                                    tracing::info!("Error on {} logout", local_login_name_session())
                                }
                            }
                        }
                        mobile_nav_open.set(false);
                        let navigator = use_navigator();
                        navigator.push(Route::Home {});
                    },
                    if is_signed_in(&snap_local_login_name_session) {
                        {t!("navbar-sign-out")}
                    } else {
                        {t!("navbar-sign-in")}
                    }
                }
            }

            Outlet::<Route> {}

            // ── Footer ────────────────────────────────────────────────────────────
            footer { class: "app-footer",
                div { class: "app-footer-inner",
                    // Brand
                    div { class: "app-footer-brand",
                        span { class: "app-footer-icon", "⚔️" }
                        span { class: "app-footer-name", "dx-rpg" }
                        span { class: "app-footer-version", {concat!("v", env!("CARGO_PKG_VERSION"))} }
                    }
                    // About
                    div { class: "app-footer-section",
                        span { class: "app-footer-section-title", {t!("footer-about")} }
                        a {
                            href: "https://github.com/r0nd0ud0u/dx-rpg",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "GitHub"
                        }
                        span { class: "app-footer-sep", "·" }
                        a {
                            href: "https://github.com/r0nd0ud0u/lib-rpg",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            {t!("footer-lib-rpg-engine")}
                        }
                        span { class: "app-footer-sep", "·" }
                        a {
                            href: "https://dioxuslabs.com",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            {t!("footer-built-with-dioxus")}
                        }
                        span { "⚡ Rust + WASM" }
                    }
                    // Contact
                    div { class: "app-footer-section",
                        span { class: "app-footer-section-title", {t!("footer-contact")} }
                        a {
                            href: "https://github.com/r0nd0ud0u/dx-rpg/issues",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            {t!("footer-report-issue")}
                        }
                        span { class: "app-footer-sep", "·" }
                        a {
                            href: "https://github.com/r0nd0ud0u/dx-rpg/discussions",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            {t!("footer-discussions")}
                        }
                    }
                }
            }
        } // end page-layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_link_visible_only_for_admin() {
        assert!(is_admin_link_visible(ADMIN.as_str()));
        assert!(!is_admin_link_visible("someone-else"));
        assert!(!is_admin_link_visible(DISCONNECTED_USER.as_str()));
    }

    #[test]
    fn quit_visible_only_while_running() {
        assert!(is_quit_visible(&GamePhase::Running));
        assert!(!is_quit_visible(&GamePhase::Default));
        assert!(!is_quit_visible(&GamePhase::InitGame));
        assert!(!is_quit_visible(&GamePhase::Loading));
        assert!(!is_quit_visible(&GamePhase::Overworld));
        assert!(!is_quit_visible(&GamePhase::Ended));
    }

    #[test]
    fn signed_in_is_the_inverse_of_disconnected_placeholder() {
        assert!(is_signed_in("some-user"));
        assert!(!is_signed_in(DISCONNECTED_USER.as_str()));
    }
}

use crate::{
    common::DISCONNECTED_USER,
    websocket_handler::{NO_CLIENT_ID, msg_from_client::send_disconnect_from_server_data},
};
use dioxus::{logger::tracing, prelude::*};
use dioxus_i18n::t;
use lib_rpg::server::server_manager::{GamePhase, ServerData};

use crate::{
    audio::{self, MusicTrack},
    auth_manager::server_fn::{change_password, get_use_password, logout},
    board_game_components::{debug_console::DebugConsole, tutorial::HowToPlayDialog},
    common::{
        ADMIN, ConnectionStatus, CtxAppLang, CtxAudioSettings, CtxConnectionLatency,
        CtxConnectionStatus, CtxSyncedInsecureCerts, CtxSyncedServerUrl, CtxTutorialSeen, Route,
    },
    components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
            AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
        },
        button::{Button, ButtonVariant},
        input::Input,
        sidebar::{Sidebar, SidebarTrigger},
    },
    game_channel::GameChannel,
    sfx_cue::Sfx,
    websocket_handler::{
        event::ClientEvent, msg_from_client::send_disconnect_from_server_data as send_quit,
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

/// Ends the current session; the caller navigates home either way.
///
/// The local session is cleared whatever the server answers. The username lives in local
/// storage, so the client still believes it is signed in after the server session has gone
/// (app update, server restart, expired cookie) — exactly when `logout()` fails, and
/// gating on it left the player stuck. Offline there is no server to notify at all, and
/// `go_online()` is needed so the login page can reach one again.
// `mut socket` is only for `go_online()`, cfg'd out of the server build.
#[cfg_attr(feature = "server", allow(unused_mut))]
async fn sign_out(
    mut socket: GameChannel,
    mut login_name_session: Signal<String>,
    mut login_id_session: Signal<i64>,
) {
    let name = login_name_session();
    if name == *DISCONNECTED_USER {
        return;
    }
    #[cfg(not(feature = "server"))]
    let offline = socket.is_offline();
    #[cfg(feature = "server")]
    let offline = false;

    if offline {
        #[cfg(not(feature = "server"))]
        socket.go_online();
    } else {
        if let Err(e) = logout().await {
            tracing::warn!("server-side sign-out for {name} failed, clearing locally: {e}");
        }
        // Sent regardless: it drops the player from the server's roster, and a
        // server that just refused the logout is all the more likely to be holding
        // a stale entry for them.
        let _ = socket.send(ClientEvent::RequestLogOut(name.clone())).await;
    }
    tracing::info!("{name} is signed out");
    *login_name_session.write() = (*DISCONNECTED_USER).to_string();
    *login_id_session.write() = NO_CLIENT_ID;
}

/// Whether the current username represents a signed-in user (vs. the
/// disconnected placeholder), i.e. whether the sign-out label/state applies.
fn is_signed_in(username: &str) -> bool {
    username != DISCONNECTED_USER.as_str()
}

/// Whether the connection-status badge should be shown: only while signed in to a real
/// (non-offline) session — offline mode has no network backend to be up or down, and
/// there's nothing to report before sign-in either.
fn is_connection_status_visible(username: &str, is_offline: bool) -> bool {
    is_signed_in(username) && !is_offline
}

/// Bars (0-4) for the connection icon. A latency bucket, not real signal strength — no
/// cross-platform API exposes the radio. `Reconnecting` is always 0, drawn in a distinct
/// pulsing colour rather than as a weak signal.
fn connection_bars(status: ConnectionStatus, latency_ms: Option<u64>) -> u8 {
    if status == ConnectionStatus::Reconnecting {
        return 0;
    }
    match latency_ms {
        // Connected but no completed measurement yet (just (re)connected, or the last
        // ping timed out) — treat as weak rather than claiming a strength we don't know.
        None => 1,
        Some(ms) if ms < 100 => 4,
        Some(ms) if ms < 300 => 3,
        Some(ms) if ms < 600 => 2,
        Some(_) => 1,
    }
}

/// Tooltip text for the connection-signal icon.
fn connection_status_title(status: ConnectionStatus, latency_ms: Option<u64>) -> String {
    match (status, latency_ms) {
        (ConnectionStatus::Reconnecting, _) => t!("navbar-connection-reconnecting"),
        (ConnectionStatus::Connected, Some(ms)) => {
            t!("navbar-connection-latency", ms : ms.to_string())
        }
        (ConnectionStatus::Connected, None) => t!("navbar-connection-connected"),
    }
}

/// Filling wifi icon standing in for connection quality (see `connection_bars`).
/// Reconnecting pulses in the danger color via the `.reconnecting` class (main.css)
/// regardless of `bars` (always 0 in that state) so a full drop reads distinctly from
/// "connected but weak".
#[component]
fn ConnectionSignalIcon(bars: u8, reconnecting: bool) -> Element {
    let icon_class = if reconnecting {
        "navbar-connection-icon reconnecting"
    } else {
        "navbar-connection-icon"
    };
    let seg_class = |threshold: u8| if bars >= threshold { "lit" } else { "dim" };
    rsx! {
        svg {
            class: icon_class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            width: "20",
            height: "20",
            path { class: seg_class(4), d: "M1.42 9a16 16 0 0 1 21.16 0" }
            path { class: seg_class(3), d: "M5 12.55a11 11 0 0 1 14.08 0" }
            path { class: seg_class(2), d: "M8.53 16.11a6 6 0 0 1 6.95 0" }
            line {
                class: seg_class(1),
                x1: "12",
                y1: "20",
                x2: "12.01",
                y2: "20",
            }
        }
    }
}

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // contexts
    let socket = use_context::<GameChannel>();
    let local_login_name_session = use_context::<Signal<String>>();
    let local_login_id_session = use_context::<Signal<i64>>();
    let server_data = use_context::<Signal<ServerData>>();
    let mut app_lang = use_context::<CtxAppLang>().0;
    // Native clients only — see the doc comment on CtxSyncedServerUrl in common.rs for why
    // these are declared in App() and consumed here via context rather than called
    // directly with use_synced_storage in Navbar (a #[layout(...)] component).
    let mut synced_server_url = use_context::<CtxSyncedServerUrl>().0;
    let mut synced_insecure_certs = use_context::<CtxSyncedInsecureCerts>().0;
    let mut audio_settings = use_context::<CtxAudioSettings>();
    let connection_status = use_context::<CtxConnectionStatus>().0;
    let latency_ms = use_context::<CtxConnectionLatency>().0;

    // nav
    let navigator = use_navigator();

    // dialog open states — lifted here so the roots can live outside the navbar div
    let mut help_open = use_signal(|| false);
    // First session on this device: open the tutorial without being asked, and mark it
    // seen straight away — a player who dismisses it immediately has still been offered
    // it, and the ❓ button reopens it. The flag is persisted in local storage, declared
    // in App() (see CtxTutorialSeen).
    //
    // The check waits for a signed-in session (an offline game counts) rather than firing
    // on mount, for two reasons: it lands the dialog on the home page, where its first
    // step is the next thing the player does, and it keeps it out of the hydration window
    // on web — dioxus-sdk-storage serves a synced signal's *default* for the hydrating
    // render and only restores the stored value one render later, so a flag read (or
    // written) before then is the default, not what the device actually saved.
    let mut tutorial_seen = use_context::<CtxTutorialSeen>().0;
    let mut first_run_checked = use_signal(|| false);
    let mut tutorial_first_run = use_signal(|| false);
    use_effect(move || {
        // Read inside the effect: that is what subscribes it to later sign-ins.
        if !is_signed_in(&local_login_name_session()) || first_run_checked() {
            return;
        }
        first_run_checked.set(true);
        if !tutorial_seen() {
            tutorial_seen.set(true);
            tutorial_first_run.set(true);
            help_open.set(true);
        }
    });
    let mut quit_open = use_signal(|| false);
    let mut sound_settings_open = use_signal(|| false);
    let mut debug_console_open = use_signal(|| false);

    // Which background track (if any) should be playing, decided from the current
    // GamePhase. Navbar is the shared #[layout] component mounted on every route, so
    // this is the one place music transitions are decided — avoids each page having to
    // remember to stop music it started on unmount.
    let mut current_music_track: Signal<Option<MusicTrack>> = use_signal(|| None);
    use_effect(move || {
        let desired = match server_data().core_game_data.game_phase {
            GamePhase::Overworld => Some(MusicTrack::Overworld),
            GamePhase::Running => None, // silence during combat — sfx read more clearly
            _ => Some(MusicTrack::Home),
        };
        if desired != current_music_track() {
            current_music_track.set(desired);
            match desired {
                Some(track) => audio::play_music(track, audio_settings),
                None => audio::stop_music(),
            }
        }
    });

    // Potion-use sound effect. Lives here (not in GameBoard/OverworldMap) since a
    // potion can be used from either place — Navbar is the one component mounted on
    // every route. Deduped on `seq` rather than turn/round: see the doc comment on
    // `ConsumableUseResult` in lib-rpg for why (overworld uses don't advance a turn).
    let mut last_potion_seq = use_signal(|| 0u64);
    use_effect(move || {
        let consumable_use = server_data()
            .core_game_data
            .game_manager
            .game_state
            .last_consumable_use
            .clone();
        if !consumable_use.launcher_id_name.is_empty() && consumable_use.seq != last_potion_seq() {
            last_potion_seq.set(consumable_use.seq);
            audio::play_sfx(Sfx::Potion, audio_settings);
        }
    });

    // Native only, gated at render time — #[cfg] isn't supported inside rsx!.
    let mut server_settings_open = use_signal(|| false);
    // Fullscreen toggle. `is_fullscreen` is unconditional so the label can read it;
    // `use_window()` is cfg-gated. `feature = "server"` is excluded too: `dx serve
    // --platform desktop` builds the companion server with `desktop` still on, and
    // `use_window()` panics there during SSR ("Could not find context Rc<DesktopService>").
    #[cfg_attr(
        not(all(feature = "desktop", not(feature = "server"))),
        allow(unused_mut)
    )]
    let mut is_fullscreen = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(feature = "server")))]
    let desktop_window = dioxus_desktop::use_window();
    // Mobile nav drawer; the desktop controls are duplicated into it, visibility
    // CSS-gated (.navbar-desktop-group/.navbar-mobile-trigger in main.css).
    let mut mobile_nav_open = use_signal(|| false);
    // Draft state so typing doesn't write to storage on every keystroke; populated from
    // the synced values each time the dialog is opened (see the trigger button below).
    let mut server_url_draft = use_signal(String::new);
    let mut insecure_certs_draft = use_signal(|| false);
    let mut server_saved_message = use_signal(|| false);

    // Change-password dialog — only relevant while USE_PASSWORD is on (see login_page.rs
    // for why this is fetched client-side via use_effect + spawn rather than use_resource).
    let mut use_pw = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            if let Ok(v) = get_use_password().await {
                use_pw.set(v);
            }
        });
    });
    let mut change_password_open = use_signal(|| false);
    let mut old_password_draft = use_signal(String::new);
    let mut new_password_draft = use_signal(String::new);
    let mut confirm_password_draft = use_signal(String::new);
    let mut change_password_answer = use_signal(String::new);

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
                    // Help trigger. Icon-only to fit the bar, so the label lives in the
                    // tooltip/accessible name instead.
                    Button {
                        variant: ButtonVariant::Outline,
                        title: t!("help-title"),
                        aria_label: t!("help-title"),
                        onclick: move |_| {
                            tutorial_first_run.set(false);
                            help_open.set(true);
                        },
                        "❓"
                    }
                    // Sound settings trigger
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| sound_settings_open.set(true),
                        {if (audio_settings.muted)() { "🔇" } else { "🔊" }}
                    }
                    // Debug console trigger (admin only) — see debug_console.rs for why:
                    // no attached developer console on mobile.
                    if is_admin_link_visible(&snap_local_login_name_session) {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| debug_console_open.set(true),
                            {t!("navbar-debug-console")}
                        }
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
                    // Fullscreen toggle (desktop only)
                    if cfg!(all(feature = "desktop", not(feature = "server"))) {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: {
                                // `desktop_window` is non-`Copy` and the drawer's duplicate
                                // button captures it too — clone per closure.
                                #[cfg(all(feature = "desktop", not(feature = "server")))]
                                let desktop_window = desktop_window.clone();
                                move |_| {
                                    #[cfg(all(feature = "desktop", not(feature = "server")))]
                                    {
                                        let next = !is_fullscreen();
                                        desktop_window.set_fullscreen(next);
                                        is_fullscreen.set(next);
                                    }
                                }
                            },
                            {if is_fullscreen() { "🗗" } else { "🗖" }}
                        }
                    }
                    // Change-password trigger (signed-in users, only while USE_PASSWORD is on)
                    if is_signed_in(&snap_local_login_name_session) && use_pw() {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                old_password_draft.set(String::new());
                                new_password_draft.set(String::new());
                                confirm_password_draft.set(String::new());
                                change_password_answer.set(String::new());
                                change_password_open.set(true);
                            },
                            {t!("navbar-change-password")}
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
                    if is_connection_status_visible(&snap_local_login_name_session, socket.is_offline()) {
                        span { title: connection_status_title(connection_status(), latency_ms()),
                            ConnectionSignalIcon {
                                bars: connection_bars(connection_status(), latency_ms()),
                                reconnecting: connection_status() == ConnectionStatus::Reconnecting,
                            }
                        }
                    }
                    if is_signed_in(&snap_local_login_name_session) {
                        span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                    }
                    Button {
                        style: "width: 160px; box-sizing: border-box; text-align: center; white-space: nowrap;",
                        variant: if is_signed_in(&snap_local_login_name_session) { ButtonVariant::Destructive } else { ButtonVariant::Secondary },
                        onclick: move |_| async move {
                            sign_out(socket, local_login_name_session, local_login_id_session)
                                .await;
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

            // How-to-play dialog — see tutorial.rs. The game phase only decides which
            // tab it opens on.
            HowToPlayDialog {
                open: help_open,
                is_admin: is_admin_link_visible(&snap_local_login_name_session),
                phase: server_data().core_game_data.game_phase.clone(),
                first_run: tutorial_first_run(),
            }

            // Sound settings dialog — sliders/mute apply live (no draft/save step, unlike
            // the server-settings dialog above, since there's nothing unsafe about a
            // volume change taking effect immediately).
            AlertDialogRoot {
                open: sound_settings_open(),
                on_open_change: move |v| sound_settings_open.set(v),
                AlertDialogContent {
                    AlertDialogTitle { {t!("sound-settings-title")} }
                    AlertDialogDescription {
                        div { style: "text-align:left; display:flex; flex-direction:column; gap:1rem;",
                            label { style: "display:flex; align-items:center; gap:0.5rem; cursor:pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: (audio_settings.muted)(),
                                    onchange: move |e: FormEvent| {
                                        (audio_settings.muted).set(e.checked());
                                        audio::set_music_volume(audio_settings);
                                    },
                                }
                                span { {t!("sound-settings-muted")} }
                            }
                            label { style: "display:flex; align-items:center; gap:0.5rem; cursor:pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: (audio_settings.background)(),
                                    onchange: move |e: FormEvent| {
                                        (audio_settings.background).set(e.checked());
                                        audio::set_background_audio(audio_settings);
                                    },
                                }
                                span { {t!("sound-settings-background")} }
                            }
                            label { style: "display:flex; flex-direction:column; gap:0.25rem;",
                                span { {t!("sound-settings-music-volume")} }
                                Input {
                                    r#type: "range",
                                    min: "0",
                                    max: "100",
                                    value: "{(audio_settings.music_volume)()}",
                                    oninput: move |e: FormEvent| {
                                        (audio_settings.music_volume).set(e.value().parse().unwrap_or(60));
                                        audio::set_music_volume(audio_settings);
                                    },
                                }
                            }
                            label { style: "display:flex; flex-direction:column; gap:0.25rem;",
                                span { {t!("sound-settings-sfx-volume")} }
                                Input {
                                    r#type: "range",
                                    min: "0",
                                    max: "100",
                                    value: "{(audio_settings.sfx_volume)()}",
                                    oninput: move |e: FormEvent| {
                                        (audio_settings.sfx_volume).set(e.value().parse().unwrap_or(80));
                                    },
                                    onchange: move |_| audio::play_sfx(Sfx::Strike, audio_settings),
                                }
                            }
                        }
                    }
                    AlertDialogAction {
                        AlertDialogCancel { {t!("common-close")} }
                    }
                }
            }

            // Debug console dialog (admin only)
            if is_admin_link_visible(&snap_local_login_name_session) {
                DebugConsole {
                    open: debug_console_open(),
                    on_open_change: move |v| debug_console_open.set(v),
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

            // Change-password dialog
            if is_signed_in(&snap_local_login_name_session) && use_pw() {
                AlertDialogRoot {
                    open: change_password_open(),
                    on_open_change: move |v| change_password_open.set(v),
                    AlertDialogContent {
                        AlertDialogTitle { {t!("change-password-title")} }
                        AlertDialogDescription {
                            div { style: "text-align:left; display:flex; flex-direction:column; gap:0.75rem;",
                                Input {
                                    placeholder: t!("change-password-current-placeholder"),
                                    r#type: "password",
                                    value: "{old_password_draft}",
                                    oninput: move |e: FormEvent| old_password_draft.set(e.value()),
                                }
                                Input {
                                    placeholder: t!("change-password-new-placeholder"),
                                    r#type: "password",
                                    value: "{new_password_draft}",
                                    oninput: move |e: FormEvent| new_password_draft.set(e.value()),
                                }
                                Input {
                                    placeholder: t!("change-password-confirm-placeholder"),
                                    r#type: "password",
                                    value: "{confirm_password_draft}",
                                    oninput: move |e: FormEvent| confirm_password_draft.set(e.value()),
                                }
                                if !change_password_answer().is_empty() {
                                    p { class: "rpg-answer", "{change_password_answer}" }
                                }
                            }
                        }
                        AlertDialogActions {
                            AlertDialogCancel { {t!("common-cancel")} }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| async move {
                                    if new_password_draft().trim().is_empty() {
                                        change_password_answer.set(t!("change-password-empty"));
                                        return;
                                    }
                                    if new_password_draft() != confirm_password_draft() {
                                        change_password_answer.set(t!("change-password-mismatch"));
                                        return;
                                    }
                                    match change_password(
                                            local_login_name_session(),
                                            old_password_draft(),
                                            new_password_draft(),
                                            use_pw(),
                                        )
                                        .await
                                    {
                                        Ok(()) => {
                                            change_password_answer.set(t!("change-password-saved"));
                                            old_password_draft.set(String::new());
                                            new_password_draft.set(String::new());
                                            confirm_password_draft.set(String::new());
                                        }
                                        Err(e) => {
                                            change_password_answer.set(format!("{}", e.to_owned()));
                                        }
                                    }
                                },
                                {t!("change-password-save")}
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
                        tutorial_first_run.set(false);
                        help_open.set(true);
                        mobile_nav_open.set(false);
                    },
                    {t!("help-title")}
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        sound_settings_open.set(true);
                        mobile_nav_open.set(false);
                    },
                    {if (audio_settings.muted)() { "🔇" } else { "🔊" }}
                    " "
                    {t!("sound-settings-title")}
                }
                if is_admin_link_visible(&snap_local_login_name_session) {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            debug_console_open.set(true);
                            mobile_nav_open.set(false);
                        },
                        {t!("navbar-debug-console")}
                    }
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
                if cfg!(all(feature = "desktop", not(feature = "server"))) {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: {
                            #[cfg(all(feature = "desktop", not(feature = "server")))]
                            let desktop_window = desktop_window.clone();
                            move |_| {
                                #[cfg(all(feature = "desktop", not(feature = "server")))]
                                {
                                    let next = !is_fullscreen();
                                    desktop_window.set_fullscreen(next);
                                    is_fullscreen.set(next);
                                }
                                mobile_nav_open.set(false);
                            }
                        },
                        {if is_fullscreen() { "🗗" } else { "🗖" }}
                        " "
                        {t!("navbar-fullscreen-toggle")}
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
                if is_signed_in(&snap_local_login_name_session) && use_pw() {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            old_password_draft.set(String::new());
                            new_password_draft.set(String::new());
                            confirm_password_draft.set(String::new());
                            change_password_answer.set(String::new());
                            change_password_open.set(true);
                            mobile_nav_open.set(false);
                        },
                        {t!("navbar-change-password")}
                    }
                }
                if is_connection_status_visible(&snap_local_login_name_session, socket.is_offline()) {
                    span { title: connection_status_title(connection_status(), latency_ms()),
                        ConnectionSignalIcon {
                            bars: connection_bars(connection_status(), latency_ms()),
                            reconnecting: connection_status() == ConnectionStatus::Reconnecting,
                        }
                    }
                }
                if is_signed_in(&snap_local_login_name_session) {
                    span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                }
                Button {
                    variant: if is_signed_in(&snap_local_login_name_session) { ButtonVariant::Destructive } else { ButtonVariant::Secondary },
                    onclick: move |_| async move {
                        sign_out(socket, local_login_name_session, local_login_id_session)
                            .await;
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
                        span { class: "app-footer-name", "RPG Adventure" }
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

    #[test]
    fn connection_status_hidden_when_signed_out_or_offline() {
        assert!(is_connection_status_visible("some-user", false));
        assert!(!is_connection_status_visible(
            DISCONNECTED_USER.as_str(),
            false
        ));
        assert!(!is_connection_status_visible("some-user", true));
    }

    #[test]
    fn connection_bars_always_zero_while_reconnecting() {
        assert_eq!(connection_bars(ConnectionStatus::Reconnecting, None), 0);
        assert_eq!(connection_bars(ConnectionStatus::Reconnecting, Some(20)), 0);
    }

    #[test]
    fn connection_bars_scale_with_latency() {
        assert_eq!(connection_bars(ConnectionStatus::Connected, None), 1);
        assert_eq!(connection_bars(ConnectionStatus::Connected, Some(50)), 4);
        assert_eq!(connection_bars(ConnectionStatus::Connected, Some(200)), 3);
        assert_eq!(connection_bars(ConnectionStatus::Connected, Some(450)), 2);
        assert_eq!(connection_bars(ConnectionStatus::Connected, Some(900)), 1);
    }
}

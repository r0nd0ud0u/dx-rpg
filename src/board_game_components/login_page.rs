use dioxus::logger::tracing;
use dioxus::prelude::*;
// Not re-exported by dioxus::prelude, unlike `use_navigator` which produces it.
use dioxus::router::Navigator;
use dioxus_i18n::t;

use crate::auth_manager::server_fn::get_user_id;
use crate::game_channel::GameChannel;
use crate::websocket_handler::event::ClientEvent;
use crate::{
    auth_manager::server_fn::{get_use_password, login, register},
    common::{CtxDeviceToken, CtxSessionExpired, Route},
    components::{
        button::{Button, ButtonVariant},
        input::Input,
    },
};
#[cfg(not(feature = "server"))]
use crate::{
    local_channel::LOCAL_PLAYER_NAME, local_engine::list_universes,
    websocket_handler::msg_from_client::send_initialize_game,
};

/// The one message shown under the auth form. Signing in and signing up now share
/// a single field set, so only one of them can be reporting anything at a time.
#[derive(Clone, PartialEq)]
struct Feedback {
    text: String,
    is_error: bool,
}

impl Feedback {
    fn ok(text: String) -> Self {
        Self {
            text,
            is_error: false,
        }
    }

    fn err(text: String) -> Self {
        Self {
            text,
            is_error: true,
        }
    }
}

#[component]
pub fn LoginPage() -> Element {
    // contexts
    let socket = use_context::<GameChannel>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    let mut device_token = use_context::<CtxDeviceToken>().0;
    let mut session_expired = use_context::<CtxSessionExpired>().0;
    // nav
    let navigator = use_navigator();

    // One pair of fields for both actions. Signing in and creating an account ask
    // for exactly the same two things, so the previous two side-by-side cards were
    // the same form twice — which made the reader pick a branch before they had any
    // reason to, and pushed "play offline" off the bottom of a phone screen.
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut feedback: Signal<Option<Feedback>> = use_signal(|| None);

    // use_effect + spawn, not use_resource: use_resource resolves during SSR and embeds the
    // value for hydration, hitting DioxusLabs/dioxus#3583 — the page comes up unresponsive.
    // Same pattern as `is_admin_enabled` in admin_page.rs.
    let mut use_pw = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            if let Ok(v) = get_use_password().await {
                use_pw.set(v);
            }
        });
    });

    // Returns the complaint to show when the form isn't filled in, so both actions
    // refuse identically instead of drifting apart.
    let missing_fields = move || -> Option<String> {
        (username().trim().is_empty() || (use_pw() && password().trim().is_empty())).then(|| {
            if use_pw() {
                t!("login-empty-fields")
            } else {
                t!("login-empty-username")
            }
        })
    };

    // Everything that happens once the server has accepted these credentials —
    // the same sequence whether the account was just created or already existed.
    //
    // Returns false without navigating if the session can't be resolved, and
    // navigates as its very last act otherwise: a caller can still write to its
    // own signals after a `false`, but must not after a `true`, since the push
    // drops this component's scope.
    let enter_session = move |name: String, proof: String| async move {
        let Ok(sql_id) = get_user_id().await else {
            // The credentials were just accepted, so this is a server-side session
            // anomaly rather than a bad password. Previously sign-in silently stayed
            // put while sign-up carried on with a placeholder id; neither told the
            // player anything, so both now stop here and say so.
            tracing::warn!("credentials accepted but get_user_id failed — staying on login");
            return false;
        };
        session_expired.set(false);
        // The server only just handed us this proof because a real login succeeded —
        // persist it as our device_token so AddPlayer/LoginAllSessions can prove to
        // the websocket that we're actually allowed to claim this username.
        device_token.set(proof.clone());
        *local_login_id_session.write() = sql_id;
        *local_login_name_session.write() = name.clone();
        let _ = socket
            .clone()
            .send(ClientEvent::LoginAllSessions(name, sql_id, proof))
            .await;
        navigator.push(Route::Home {});
        true
    };

    rsx! {
        div { class: "home-container",
            div { class: "rotate-scale-up",
                h1 { class: "rpg-title", {t!("home-title")} }
            }
            if session_expired() {
                div { class: "session-expired-banner",
                    span { {t!("login-session-expired")} }
                    button {
                        class: "session-expired-dismiss",
                        "aria-label": t!("common-close"),
                        onclick: move |_| session_expired.set(false),
                        "×"
                    }
                }
            }
            div { class: "auth-grid",
                // --- Sign in / sign up: one form, two actions ---
                div { class: "rpg-card auth-card",
                    p { class: "auth-section-title", {t!("login-sign-in-title")} }
                    Input {
                        placeholder: t!("login-username-placeholder"),
                        r#type: "text",
                        value: "{username}",
                        oninput: move |e: FormEvent| username.set(e.value()),
                    }
                    if use_pw() {
                        Input {
                            placeholder: t!("login-password-placeholder"),
                            r#type: "password",
                            value: "{password}",
                            oninput: move |e: FormEvent| password.set(e.value()),
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| async move {
                            if let Some(message) = missing_fields() {
                                feedback.set(Some(Feedback::err(message)));
                                return;
                            }
                            tracing::info!("Attempting to log in with username: {}", username());
                            match login(username(), password(), use_pw()).await {
                                Ok(proof) => {
                                    feedback
                                        .set(
                                            Some(Feedback::ok(t!("login-success", username : username()))),
                                        );
                                    if !enter_session(username(), proof).await {
                                        feedback.set(Some(Feedback::err(t!("login-invalid-login"))));
                                    }
                                }
                                Err(e) => {
                                    tracing::info!("{}", e.to_owned());
                                    feedback.set(Some(Feedback::err(format!("{}", e.to_owned()))));
                                }
                            }
                        },
                        {t!("login-sign-in-button")}
                    }
                    // Secondary on purpose: most visits are a returning player, and the
                    // same fields above serve both. Ghost styling keeps the primary path
                    // obvious without hiding this one.
                    div { class: "auth-alt",
                        span { class: "auth-desc", {t!("login-no-account")} }
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: move |_| async move {
                                if let Some(message) = missing_fields() {
                                    feedback.set(Some(Feedback::err(message)));
                                    return;
                                }
                                if let Err(e) = register(username(), password(), use_pw()).await {
                                    tracing::info!("{}", e.to_owned());
                                    feedback.set(Some(Feedback::err(t!("login-name-taken"))));
                                    return;
                                }
                                match login(username(), password(), use_pw()).await {
                                    Ok(proof) => {
                                        feedback
                                            .set(
                                                Some(Feedback::ok(t!("login-success", username : username()))),
                                            );
                                        if !enter_session(username(), proof).await {
                                            feedback.set(Some(Feedback::err(t!("login-invalid-login"))));
                                        }
                                    }
                                    Err(e) => {
                                        tracing::info!("{}", e.to_owned());
                                        feedback.set(Some(Feedback::err(t!("login-invalid-login"))));
                                    }
                                }
                            },
                            {t!("login-sign-up-button")}
                        }
                    }
                    if let Some(message) = feedback() {
                        p {
                            class: if message.is_error { "rpg-answer-error" } else { "rpg-answer" },
                            "{message.text}"
                        }
                    }
                }
                PlayOfflineCard {}
            }
        }
    }
}

/// The universes a local game can be started in. Empty on the server build, which
/// has no local engine — harmlessly, because the list is only ever rendered after
/// the player has clicked, which is necessarily after hydration.
#[cfg(not(feature = "server"))]
fn offline_universes() -> Vec<String> {
    list_universes().unwrap_or_default()
}

#[cfg(feature = "server")]
fn offline_universes() -> Vec<String> {
    Vec::new()
}

/// Enters a local, no-server game. A no-op on the server build, where the local
/// engine doesn't exist — `cfg!()` would not do, since it is a runtime check and
/// `go_offline`/`send_initialize_game` have to resolve at compile time.
#[cfg(not(feature = "server"))]
fn start_offline_game(
    universe: String,
    socket: GameChannel,
    mut player_name: Signal<String>,
    navigator: Navigator,
) {
    spawn(async move {
        let mut socket = socket;
        socket.go_offline();
        *player_name.write() = LOCAL_PLAYER_NAME.to_owned();
        send_initialize_game(LOCAL_PLAYER_NAME, &universe, true, socket).await;
        navigator.push(Route::LobbyPage {});
    });
}

#[cfg(feature = "server")]
fn start_offline_game(_: String, _: GameChannel, _: Signal<String>, _: Navigator) {}

/// Pick a universe and jump straight into a local, no-server single-player game
/// (see `game_channel.rs`/`local_channel.rs`). Shown beside the auth card on this
/// page, and as the *only* action on `Home` once the session is already offline —
/// where creating or joining a server is not something an offline player can do.
///
/// One component, rendering the same markup on the server and the client, with only
/// the *behaviour* `#[cfg]`-split into the two free functions above. It used to be
/// two `#[cfg]`-swapped components, the server one returning `rsx! {}` — which made
/// this card invisible in the browser: the fullstack client hydrates onto the
/// markup the server produced, binding to nodes that are already there, and never
/// inserts one the server left out. The card was compiled into the wasm and simply
/// never appeared. Anything conditional here must therefore differ only *after*
/// hydration (as `offline_started` does), never in the first render.
#[component]
pub fn PlayOfflineCard() -> Element {
    let socket = use_context::<GameChannel>();
    let player_name = use_context::<Signal<String>>();
    let navigator = use_navigator();
    let universes = use_signal(offline_universes);
    // The universe dropdown only appears after this is clicked, so starting offline play
    // is a single, unambiguous button rather than one button per universe (the previous
    // design) cluttering the login screen.
    let mut offline_started = use_signal(|| false);

    rsx! {
        div { class: "rpg-card auth-card",
            p { class: "auth-section-title", {t!("login-offline-title")} }
            p { class: "auth-desc", {t!("login-offline-hint")} }
            if !offline_started() {
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| offline_started.set(true),
                    {t!("login-offline-start-button")}
                }
            } else {
                select {
                    class: "lobby-select",
                    "aria-label": t!("login-offline-choose-universe-label"),
                    value: "",
                    onchange: move |e: FormEvent| {
                        let universe = e.value();
                        if !universe.is_empty() {
                            start_offline_game(universe, socket, player_name, navigator);
                        }
                    },
                    option { value: "", {t!("login-offline-choose-universe-option")} }
                    for universe in universes() {
                        option { value: "{universe}", "{universe}" }
                    }
                }
            }
        }
    }
}

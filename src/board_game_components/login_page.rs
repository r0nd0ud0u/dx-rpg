use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus::{fullstack::UseWebsocket, logger::tracing};

use crate::auth_manager::server_fn::get_user_id;
use crate::websocket_handler::NO_CLIENT_ID;
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::{
    auth_manager::server_fn::{get_use_password, login, register},
    common::Route,
    components::{
        button::{Button, ButtonVariant},
        input::Input,
    },
};

#[component]
pub fn LoginPage() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    // nav
    let navigator = use_navigator();
    // logon
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut logon_answer = use_signal(String::new);
    let set_username = move |e: FormEvent| username.set(e.value());
    let set_password = move |e: FormEvent| password.set(e.value());
    // register
    let mut register_name = use_signal(String::new);
    let mut register_password = use_signal(String::new);
    let mut register_answer = use_signal(String::new);
    let set_register = move |e: FormEvent| register_name.set(e.value());
    let set_register_pw = move |e: FormEvent| register_password.set(e.value());

    // fetch USE_PASSWORD flag from server
    let use_pw_res = use_resource(|| async { get_use_password().await.unwrap_or(false) });
    let use_pw = use_pw_res().unwrap_or(false);

    rsx! {
        div { class: "home-container",
            div { class: "rotate-scale-up",
                h1 { class: "rpg-title", "⚔️ RPG Adventure" }
            }
            div { class: "auth-grid",
                // --- Sign in card ---
                div { class: "rpg-card auth-card",
                    p { class: "auth-section-title", "Sign In" }
                    Input {
                        placeholder: "Your username",
                        r#type: "text",
                        value: "{username}",
                        oninput: set_username,
                    }
                    if use_pw {
                        Input {
                            placeholder: "Password",
                            r#type: "password",
                            value: "{password}",
                            oninput: set_password,
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| async move {
                            tracing::info!("Attempting to log in with username: {}", username());
                            match login(username(), password(), use_pw).await {
                                Ok(()) => {
                                    logon_answer.set(format!("{} logged in", username()));
                                    match get_user_id().await {
                                        Ok(sql_id) => {
                                            *local_login_id_session.write() = sql_id;
                                            *local_login_name_session.write() = username();
                                            let _ = socket
                                                .clone()
                                                .send(ClientEvent::LoginAllSessions(username(), sql_id))
                                                .await;
                                            navigator.push(Route::Home {});
                                            sql_id
                                        }
                                        Err(e) => {
                                            tracing::info!("{}", e.to_owned());
                                            -1
                                        }
                                    };
                                }
                                Err(e) => {
                                    tracing::info!("{}", e.to_owned());
                                    logon_answer.set(format!("{}", e.to_owned()));
                                }
                            }
                        },
                        "Sign In →"
                    }
                    if !logon_answer().is_empty() {
                        p { class: "rpg-answer", "{logon_answer}" }
                    }
                }
                // --- Sign up card ---
                div { class: "rpg-card auth-card",
                    p { class: "auth-section-title", "Create Account" }
                    Input {
                        placeholder: "Choose a username",
                        r#type: "text",
                        value: "{register_name}",
                        oninput: set_register,
                    }
                    if use_pw {
                        Input {
                            placeholder: "Choose a password",
                            r#type: "password",
                            value: "{register_password}",
                            oninput: set_register_pw,
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| async move {
                            match register(register_name(), register_password(), use_pw).await {
                                Ok(()) => {
                                    match login(register_name(), register_password(), use_pw).await {
                                        Ok(()) => {
                                            *local_login_name_session.write() = register_name();
                                            *local_login_id_session.write() = (get_user_id().await)
                                                .unwrap_or(NO_CLIENT_ID);
                                            navigator.push(Route::Home {});
                                        }
                                        Err(e) => {
                                            tracing::info!("{}", e.to_owned());
                                            register_answer.set("Invalid login".to_owned());
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::info!("{}", e.to_owned());
                                    register_answer.set("This name is already taken.".to_owned());
                                }
                            }
                        },
                        "Sign Up →"
                    }
                    if !register_answer().is_empty() {
                        p { class: "rpg-answer-error", "{register_answer}" }
                    }
                }
            }
        }
    }
}

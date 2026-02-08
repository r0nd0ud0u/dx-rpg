use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus::{fullstack::UseWebsocket, logger::tracing};

use crate::auth_manager::server_fn::get_user_id;
use crate::components::label::Label;
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::{
    auth_manager::server_fn::{login, register},
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
    let mut logon_answer = use_signal(String::new);
    let set_username = move |e: FormEvent| {
        username.set(e.value());
    };
    // register
    let mut register_name = use_signal(String::new);
    let mut register_answer = use_signal(String::new);
    let set_register = move |e: FormEvent| {
        register_name.set(e.value());
    };

    rsx! {
        div { class: "home-container",
            h1 { "Welcome to the RPG game!" }
            div {
                display: "grid",
                flex: "1 1 0%",
                grid_auto_rows: "min-content",
                gap: "1.5rem",
                padding: "0 1rem",
                div { display: "grid", gap: "0.75rem",
                    Label { html_for: "sheet-demo-name", "Sign in" }
                    Input {
                        placeholder: "Type your username",
                        r#type: "text",
                        value: "{username}",
                        oninput: set_username,
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| async move {
                            match login(username(), "".to_owned(), false).await {
                                Ok(()) => {
                                    logon_answer.set(format!("{} is logged", username()));
                                    // set local storage for login
                                    match get_user_id().await {
                                        Ok(sql_id) => {
                                            // set local storage
                                            *local_login_id_session.write() = sql_id;
                                            *local_login_name_session.write() = username();
                                            // notify server via websocket
                                            let _ = socket
                                                // change page
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
                        "Connexion"
                    }
                    Label { html_for: "sheet-demo-name", "{logon_answer}" }
                    Label { html_for: "sheet-demo-name", "Sign up" }
                    Input {
                        placeholder: "Choose your username",
                        r#type: "text",
                        value: "{register_name}",
                        oninput: set_register,
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| async move {
                            match register(register_name(), "".to_owned(), false).await {
                                Ok(()) => {
                                    match login(register_name(), "".to_owned(), false).await {
                                        Ok(()) => {
                                            // local storage for login
                                            *local_login_name_session.write() = register_name();
                                            *local_login_id_session.write() = (get_user_id().await) // TODO default value -1
                                                // change page
                                                .unwrap_or(-1);
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
                                    register_answer.set("This name is already used.".to_owned());
                                }
                            }
                        },
                        "Sign up"
                    }
                    Label { html_for: "sheet-demo-name", "{register_answer}" }
                }
            }
        }
    }
}

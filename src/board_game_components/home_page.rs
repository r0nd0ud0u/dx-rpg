use dioxus::prelude::*;
use dioxus_primitives::label::Label;

use crate::{
    auth::server_fn::login,
    board_game_components::common_comp::ButtonLink,
    common::{Route, USER_NAME},
    components::{
        button::{Button, ButtonVariant},
        input::Input,
    },
};

/// Home page
#[component]
pub fn Home() -> Element {
    let mut username = use_signal(|| String::new());
    let mut logon_answer = use_signal(|| String::new());
    let set_username = move |e: FormEvent| {
        username.set(e.value());
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
                    Label { html_for: "sheet-demo-name", "Name" }
                    Input {
                        placeholder: "Type a message",
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
                                    *USER_NAME.write() = username();
                                }
                                Err(e) => {
                                    logon_answer.set(format!("{}", e));
                                }
                            }
                        },
                        "Log on"
                    }
                    Label { html_for: "sheet-demo-name", "{logon_answer}" }
                }
            }

            ButtonLink {
                target: Route::CreateServer {}.into(),
                name: "Create Server".to_string(),
            }
            ButtonLink {
                target: Route::JoinOngoingGame {}.into(),
                name: "Join game".to_string(),
            }
        }
    }
}

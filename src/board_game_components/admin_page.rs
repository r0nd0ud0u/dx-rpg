use dioxus::logger::tracing;
use dioxus::prelude::*;
use dioxus_primitives::label::Label;

use crate::{
    auth_manager::server_fn::delete_user,
    components::{
        button::{Button, ButtonVariant},
        input::Input,
    },
};

#[component]
pub fn AdminPage() -> Element {
    // delete
    let mut delete_name = use_signal(String::new);
    let mut delete_answer = use_signal(String::new);
    let set_delete = move |e: FormEvent| {
        delete_name.set(e.value());
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
                    Label { html_for: "sheet-demo-name", "Delete user" }
                    Input {
                        placeholder: "Delete user",
                        r#type: "text",
                        value: "{delete_name}",
                        oninput: set_delete,
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| async move {
                            match delete_user(delete_name(), "".to_owned(), false).await {
                                Ok(()) => {
                                    delete_answer.set("User deleted.".to_owned());
                                }
                                Err(e) => {
                                    tracing::info!("{}", e.to_owned());
                                    delete_answer.set("This name cannot be deleted.".to_owned());
                                }
                            }
                        },
                        "Delete user"
                    }
                    Label { html_for: "sheet-demo-name", "{delete_answer}" }
                
                }
            }
        }
    }
}

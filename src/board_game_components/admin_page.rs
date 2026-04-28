use dioxus::logger::tracing;
use dioxus::prelude::*;

use crate::{
    auth_manager::server_fn::delete_user,
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        label::Label,
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
            h2 { class: "rpg-title", "🛡️ Admin Panel" }
            div { class: "admin-card",
                p { class: "admin-section-title", "🗑️ Delete User" }
                Label {
                    html_for: "admin-delete",
                    color: "var(--rpg-text-muted)",
                    font_size: "0.82rem",
                    "Username to delete"
                }
                Input {
                    placeholder: "Enter username…",
                    r#type: "text",
                    value: "{delete_name}",
                    oninput: set_delete,
                }
                Button {
                    variant: ButtonVariant::Destructive,
                    onclick: move |_| async move {
                        match delete_user(delete_name(), "".to_owned(), false).await {
                            Ok(()) => {
                                delete_answer.set("✅ User deleted.".to_owned());
                            }
                            Err(e) => {
                                tracing::info!("{}", e.to_owned());
                                delete_answer.set("❌ This name cannot be deleted.".to_owned());
                            }
                        }
                    },
                    "Delete User"
                }
                if !delete_answer().is_empty() {
                    p { class: if delete_answer().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                        "{delete_answer}"
                    }
                }
            }
        }
    }
}

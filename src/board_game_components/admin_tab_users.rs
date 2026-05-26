use dioxus::logger::tracing;
use dioxus::prelude::*;

use crate::{
    auth_manager::server_fn::{AdminUserInfo, admin_list_users, delete_user},
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        label::Label,
    },
};

#[component]
pub fn AdminUsersTab() -> Element {
    let mut users: Signal<Vec<AdminUserInfo>> = use_signal(Vec::new);
    let mut delete_name = use_signal(String::new);
    let mut delete_answer = use_signal(String::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            match admin_list_users().await {
                Ok(u) => {
                    users.set(u);
                    loading.set(false);
                }
                Err(e) => tracing::error!("admin_list_users: {e}"),
            }
        });
    });

    rsx! {
        div { class: "admin-card",
            p { class: "admin-section-title", "📋 All Users" }
            if loading() {
                p { style: "color:var(--rpg-text-muted);", "Loading…" }
            } else if users().is_empty() {
                p { style: "color:var(--rpg-text-muted);", "No users found." }
            } else {
                table { class: "admin-table",
                    thead {
                        tr {
                            th { "Username" }
                            th { "Connected" }
                            th { "Saves" }
                        }
                    }
                    tbody {
                        for user in users() {
                            tr {
                                td { "{user.username}" }
                                td {
                                    if user.is_connected {
                                        span { style: "color:var(--rpg-success-light);",
                                            "🟢"
                                        }
                                    } else {
                                        span { style: "color:var(--rpg-text-muted);",
                                            "⚫"
                                        }
                                    }
                                }
                                td { "{user.nb_saves}" }
                            }
                        }
                    }
                }
            }
        }

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
                oninput: move |e: FormEvent| delete_name.set(e.value()),
            }
            Button {
                variant: ButtonVariant::Destructive,
                onclick: move |_| async move {
                    match delete_user(delete_name(), "".to_owned(), false).await {
                        Ok(()) => {
                            delete_answer.set("✅ User deleted.".to_owned());
                            if let Ok(u) = admin_list_users().await {
                                users.set(u);
                            }
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

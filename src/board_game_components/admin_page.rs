use dioxus::logger::tracing;
use dioxus::prelude::*;

use crate::{
    auth_manager::server_fn::{
        AdminCharacterInfo, AdminScenarioInfo, AdminUserInfo, admin_list_characters,
        admin_list_scenarios, admin_list_users, delete_user, is_admin_enabled,
    },
    common::PATH_IMG,
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        label::Label,
    },
};

#[derive(Clone, PartialEq)]
enum AdminTab {
    Users,
    Scenarios,
    Characters,
}

#[component]
pub fn AdminPage() -> Element {
    let mut admin_enabled = use_signal(|| true);
    let mut tab = use_signal(|| AdminTab::Users);

    // Check if admin panel is enabled
    use_effect(move || {
        spawn(async move {
            if let Ok(enabled) = is_admin_enabled().await {
                admin_enabled.set(enabled);
            }
        });
    });

    if !admin_enabled() {
        return rsx! {
            div { class: "home-container",
                h2 { class: "rpg-title", "🛡️ Admin Panel" }
                p { class: "rpg-subtitle", "The admin panel is disabled." }
            }
        };
    }

    rsx! {
        div { class: "admin-page-container",
            h2 { class: "rpg-title", "🛡️ Admin Panel" }

            div { class: "admin-tabs",
                button {
                    class: if tab() == AdminTab::Users { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Users),
                    "👤 Users"
                }
                button {
                    class: if tab() == AdminTab::Scenarios { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Scenarios),
                    "📜 Scenarios"
                }
                button {
                    class: if tab() == AdminTab::Characters { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Characters),
                    "🧙 Characters"
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
            }
        }
    }
}

// ─── Users Tab ───────────────────────────────────────────────────────────────

#[component]
fn AdminUsersTab() -> Element {
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

// ─── Scenarios Tab ───────────────────────────────────────────────────────────

#[component]
fn AdminScenariosTab() -> Element {
    let mut scenarios: Signal<Vec<AdminScenarioInfo>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            match admin_list_scenarios().await {
                Ok(mut s) => {
                    s.sort_by_key(|sc| sc.level);
                    scenarios.set(s);
                    loading.set(false);
                }
                Err(e) => tracing::error!("admin_list_scenarios: {e}"),
            }
        });
    });

    rsx! {
        div { class: "admin-full-card",
            p { class: "admin-section-title", "📜 All Scenarios" }
            if loading() {
                p { style: "color:var(--rpg-text-muted);", "Loading…" }
            } else if scenarios().is_empty() {
                p { style: "color:var(--rpg-text-muted);", "No scenarios found." }
            } else {
                table { class: "admin-table",
                    thead {
                        tr {
                            th { class: "col-level", "Lvl" }
                            th { class: "col-universe", "Universe" }
                            th { class: "col-name", "Name" }
                            th { class: "col-bosses", "Bosses" }
                            th { class: "col-description", "Description" }
                            th { class: "col-file", "File" }
                        }
                    }
                    tbody {
                        for scenario in scenarios() {
                            tr {
                                td { class: "col-level",
                                    span { class: "scenario-chip", "{scenario.level}" }
                                }
                                td { class: "col-universe", "{scenario.universe}" }
                                td {
                                    class: "col-name",
                                    style: "font-weight:600;",
                                    "{scenario.name}"
                                }
                                td { class: "col-bosses", "{scenario.nb_bosses}" }
                                td { class: "col-description", "{scenario.description}" }
                                td { class: "col-file", "{scenario.file_name}" }
                            }
                        }
                    }
                }
            }
            p { style: "color:var(--rpg-text-muted); font-size:.78rem; margin-top:12px;",
                "ℹ️ To add, edit or delete scenarios, modify the JSON files in offlines/scenarios/ and restart the server."
            }
        }
    }
}

// ─── Characters Tab ───────────────────────────────────────────────────────────

#[component]
fn AdminCharactersTab() -> Element {
    let mut characters: Signal<Vec<AdminCharacterInfo>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            match admin_list_characters().await {
                Ok(c) => {
                    characters.set(c);
                    loading.set(false);
                }
                Err(e) => tracing::error!("admin_list_characters: {e}"),
            }
        });
    });

    rsx! {
        div { class: "admin-full-card",
            p { class: "admin-section-title", "🧙 All Hero Characters" }
            if loading() {
                p { style: "color:var(--rpg-text-muted);", "Loading…" }
            } else if characters().is_empty() {
                p { style: "color:var(--rpg-text-muted);", "No characters found." }
            } else {
                div { class: "admin-char-grid",
                    for c in characters() {
                        div { class: "admin-char-card",
                            // Portrait + identity
                            div { class: "admin-char-header",
                                img {
                                    class: "admin-char-portrait",
                                    src: format!("{}/{}.png", PATH_IMG, c.photo_name),
                                    alt: "{c.db_full_name}",
                                }
                                div { class: "admin-char-identity",
                                    span { class: "admin-char-name", "{c.db_full_name}" }
                                    div { class: "admin-char-badges",
                                        span { class: "admin-char-class", "{c.class}" }
                                        span { class: "admin-char-level", "Lv {c.level}" }
                                    }
                                }
                            }
                            // Description
                            if !c.description.is_empty() {
                                p { class: "admin-char-desc", "{c.description}" }
                            }
                            // Stats table
                            div { class: "admin-char-stats",
                                {
                                    let mut sorted_stats: Vec<(String, (u64, u64))> =
                                        c.stats.into_iter().collect();
                                    sorted_stats.sort_by(|a, b| a.0.cmp(&b.0));
                                    rsx! {
                                        for (stat_name, (cur, max)) in sorted_stats {
                                            div { class: "admin-char-stat-row",
                                                span { class: "admin-char-stat-name", "{stat_name}" }
                                                span { class: "admin-char-stat-val", "{cur} / {max}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

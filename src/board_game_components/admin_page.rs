use dioxus::logger::tracing;
use dioxus::prelude::*;

use crate::{
    auth_manager::server_fn::{
        AdminCharacterInfo, AdminScenarioInfo, AdminUserInfo, AttackFormData, CharacterFormData,
        ScenarioDetail, ScenarioLootItem, StatEntry, admin_delete_attack, admin_delete_equipment,
        admin_get_attack_form, admin_get_attack_json, admin_get_character_form,
        admin_get_character_json, admin_get_equipment_json, admin_list_attacks, admin_list_bosses,
        admin_list_characters, admin_list_equipment_categories, admin_list_equipment_items,
        admin_list_equipment_types, admin_list_scenarios, admin_list_users, admin_save_attack_form,
        admin_save_attack_json, admin_save_character_form, admin_save_character_json,
        admin_save_equipment_json, delete_scenario_json, delete_user, get_scenario_detail,
        is_admin_enabled, list_universes_server, save_scenario_detail,
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
    Equipment,
}

#[component]
pub fn AdminPage() -> Element {
    let mut admin_enabled = use_signal(|| true);
    let mut tab = use_signal(|| AdminTab::Users);

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
                button {
                    class: if tab() == AdminTab::Equipment { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set(AdminTab::Equipment),
                    "🔧 Equipment"
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
                AdminTab::Equipment => rsx! {
                    AdminEquipmentTab {}
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

#[derive(Clone, PartialEq)]
enum ScenarioEditMode {
    None,
    Edit(String),
    New,
}

#[component]
fn AdminScenariosTab() -> Element {
    let mut scenarios: Signal<Vec<AdminScenarioInfo>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut selected_universe = use_signal(String::new);
    let mut universes_resource = use_resource(list_universes_server);
    let mut edit_mode = use_signal(|| ScenarioEditMode::None);
    let mut edit_file_stem = use_signal(String::new);
    let mut edit_name = use_signal(String::new);
    let mut edit_description = use_signal(String::new);
    let mut edit_level = use_signal(|| "1".to_string());
    let mut edit_bosses = use_signal(String::new);
    let mut edit_loots: Signal<Vec<ScenarioLootItem>> = use_signal(Vec::new);
    let mut feedback = use_signal(String::new);
    let mut confirm_delete = use_signal(String::new);

    use_effect(move || {
        let u = selected_universe();
        if u.is_empty() {
            loading.set(false);
            scenarios.set(Vec::new());
            return;
        }
        loading.set(true);
        spawn(async move {
            match admin_list_scenarios().await {
                Ok(mut s) => {
                    s.retain(|sc| sc.universe == u);
                    s.sort_by_key(|sc| sc.level);
                    scenarios.set(s);
                    loading.set(false);
                }
                Err(e) => tracing::error!("admin_list_scenarios: {e}"),
            }
        });
    });

    let universes = universes_resource
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    rsx! {
        div { class: "admin-card",
            p { class: "admin-section-title", "🌐 Select Universe" }
            select {
                class: "admin-select",
                value: "{selected_universe}",
                onchange: move |e| {
                    selected_universe.set(e.value());
                    edit_mode.set(ScenarioEditMode::None);
                    feedback.set(String::new());
                    confirm_delete.set(String::new());
                },
                option { value: "", "— choose a universe —" }
                for u in &universes {
                    option { value: "{u}", "{u}" }
                }
            }
        }

        if !selected_universe().is_empty() {
            div { class: "admin-full-card",
                p { class: "admin-section-title", "📜 Scenarios — {selected_universe}" }

                if loading() {
                    p { style: "color:var(--rpg-text-muted);", "Loading…" }
                } else if scenarios().is_empty() {
                    p { style: "color:var(--rpg-text-muted);",
                        "No scenarios found for this universe."
                    }
                } else {
                    table { class: "admin-table",
                        thead {
                            tr {
                                th { class: "col-level", "Lvl" }
                                th { class: "col-name", "Name" }
                                th { class: "col-bosses", "Bosses" }
                                th { class: "col-description", "Description" }
                                th { class: "col-file", "File" }
                                th { "Actions" }
                            }
                        }
                        tbody {
                            for scenario in scenarios() {
                                {
                                    let file_stem = scenario
                                        .file_name
                                        .trim_end_matches(".json")
                                        .rsplit('/')
                                        .next()
                                        .unwrap_or(&scenario.file_name)
                                        .to_owned();
                                    let fs_clone = file_stem.clone();
                                    let fs_del = file_stem.clone();
                                    let univ = selected_universe();
                                    let univ_del = univ.clone();
                                    let is_confirm = confirm_delete() == file_stem;
                                    rsx! {
                                        tr {
                                            td { class: "col-level",
                                                span { class: "scenario-chip", "{scenario.level}" }
                                            }
                                            td { class: "col-name", style: "font-weight:600;", "{scenario.name}" }
                                            td { class: "col-bosses", "{scenario.nb_bosses}" }
                                            td { class: "col-description", "{scenario.description}" }
                                            td { class: "col-file", "{scenario.file_name}" }
                                            td {
                                                div { style: "display:flex;gap:6px;",
                                                    Button {
                                                        variant: ButtonVariant::Secondary,
                                                        onclick: move |_| {
                                                            let u = univ.clone();
                                                            let fs = fs_clone.clone();
                                                            let fs_for_mode = fs.clone();
                                                            feedback.set(String::new());
                                                            spawn(async move {
                                                                match get_scenario_detail(u, fs.clone()).await {
                                                                    Ok(detail) => {
                                                                        edit_name.set(detail.name);
                                                                        edit_description.set(detail.description);
                                                                        edit_level.set(detail.level.to_string());
                                                                        edit_bosses.set(detail.boss_patterns_text);
                                                                        edit_loots.set(detail.loots);
                                                                        edit_file_stem.set(fs);
                                                                        edit_mode.set(ScenarioEditMode::Edit(fs_for_mode));
                                                                    }
                                                                    Err(e) => feedback.set(format!("❌ {e}")),
                                                                }
                                                            });
                                                        },
                                                        "✏️ Edit"
                                                    }
                                                    if is_confirm {
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: move |_| {
                                                                let u = univ_del.clone();
                                                                let fs = fs_del.clone();
                                                                spawn(async move {
                                                                    match delete_scenario_json(u.clone(), fs).await {
                                                                        Ok(()) => {
                                                                            feedback.set("✅ Deleted.".to_owned());
                                                                            confirm_delete.set(String::new());
                                                                            edit_mode.set(ScenarioEditMode::None);
                                                                            if let Ok(mut s) = admin_list_scenarios().await {
                                                                                s.retain(|sc| sc.universe == u);
                                                                                s.sort_by_key(|sc| sc.level);
                                                                                scenarios.set(s);
                                                                            }
                                                                        }
                                                                        Err(e) => {
                                                                            feedback.set(format!("❌ {e}"));
                                                                            confirm_delete.set(String::new());
                                                                        }
                                                                    }
                                                                });
                                                            },
                                                            "⚠️ Confirm"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Secondary,
                                                            onclick: move |_| confirm_delete.set(String::new()),
                                                            "Cancel"
                                                        }
                                                    } else {
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: {
                                                                let fs = file_stem.clone();
                                                                move |_| confirm_delete.set(fs.clone())
                                                            },
                                                            "🗑️ Delete"
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

                if edit_mode() == ScenarioEditMode::None {
                    div { style: "margin-top:12px;",
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                edit_file_stem.set(String::new());
                                edit_name.set(String::new());
                                edit_description.set(String::new());
                                edit_level.set("1".to_string());
                                edit_bosses.set(String::new());
                                edit_loots.set(Vec::new());
                                edit_mode.set(ScenarioEditMode::New);
                                feedback.set(String::new());
                            },
                            "➕ Add Scenario"
                        }
                    }
                }
            }

            if edit_mode() != ScenarioEditMode::None {
                div { class: "admin-full-card",
                    p { class: "admin-section-title",
                        if edit_mode() == ScenarioEditMode::New {
                            "➕ New Scenario"
                        } else {
                            "✏️ Edit Scenario"
                        }
                    }

                    if edit_mode() == ScenarioEditMode::New {
                        Label {
                            html_for: "scenario-stem",
                            color: "var(--rpg-text-muted)",
                            font_size: "0.82rem",
                            "File stem (e.g. stage_11)"
                        }
                        Input {
                            placeholder: "stage_11",
                            r#type: "text",
                            value: "{edit_file_stem}",
                            oninput: move |e: FormEvent| edit_file_stem.set(e.value()),
                        }
                    }

                    Label {
                        html_for: "scenario-name",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        "Name"
                    }
                    Input {
                        placeholder: "Scenario name",
                        r#type: "text",
                        value: "{edit_name}",
                        oninput: move |e: FormEvent| edit_name.set(e.value()),
                    }
                    Label {
                        html_for: "scenario-desc",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        "Description"
                    }
                    Input {
                        placeholder: "Describe the scenario…",
                        r#type: "text",
                        value: "{edit_description}",
                        oninput: move |e: FormEvent| edit_description.set(e.value()),
                    }
                    Label {
                        html_for: "scenario-level",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        "Level"
                    }
                    Input {
                        placeholder: "1",
                        r#type: "number",
                        value: "{edit_level}",
                        oninput: move |e: FormEvent| edit_level.set(e.value()),
                    }
                    Label {
                        html_for: "scenario-bosses",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        "Bosses (one per line — \"BossName\" or \"BossName: 0, 1, 2\")"
                    }
                    textarea {
                        class: "admin-json-textarea",
                        rows: "4",
                        placeholder: "Gobelin Eclaireur\nAngmar: 0, 1",
                        value: "{edit_bosses}",
                        oninput: move |e: FormEvent| edit_bosses.set(e.value()),
                    }

                    Label {
                        html_for: "scenario-loots",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        "Loots"
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;",
                        for (idx, loot) in edit_loots().iter().enumerate() {
                            {
                                let loot = loot.clone();
                                let idx_rm = idx;
                                rsx! {
                                    div { class: "loot-row",
                                        Input {
                                            placeholder: "Name",
                                            r#type: "text",
                                            value: "{loot.name}",
                                            oninput: move |e: FormEvent| {
                                                let mut loots = edit_loots();
                                                if let Some(item) = loots.get_mut(idx) {
                                                    item.name = e.value();
                                                }
                                                edit_loots.set(loots);
                                            },
                                        }
                                        select {
                                            class: "admin-select loot-select",
                                            value: "{loot.kind}",
                                            onchange: move |e| {
                                                let mut loots = edit_loots();
                                                if let Some(item) = loots.get_mut(idx) {
                                                    item.kind = e.value();
                                                }
                                                edit_loots.set(loots);
                                            },
                                            option { value: "Equipment", selected: loot.kind == "Equipment", "Equipment" }
                                            option { value: "Consumable", selected: loot.kind == "Consumable", "Consumable" }
                                            option { value: "Material", selected: loot.kind == "Material", "Material" }
                                            option { value: "Currency", selected: loot.kind == "Currency", "Currency" }
                                        }
                                        select {
                                            class: "admin-select loot-select",
                                            value: "{loot.rank}",
                                            onchange: move |e| {
                                                let mut loots = edit_loots();
                                                if let Some(item) = loots.get_mut(idx) {
                                                    item.rank = e.value();
                                                }
                                                edit_loots.set(loots);
                                            },
                                            option { value: "Common", selected: loot.rank == "Common", "Common" }
                                            option { value: "Intermediate", selected: loot.rank == "Intermediate", "Intermediate" }
                                            option { value: "Advanced", selected: loot.rank == "Advanced", "Advanced" }
                                        }
                                        Input {
                                            placeholder: "Lvl",
                                            r#type: "number",
                                            value: "{loot.level}",
                                            oninput: move |e: FormEvent| {
                                                let mut loots = edit_loots();
                                                if let Some(item) = loots.get_mut(idx) {
                                                    item.level = e.value().trim().parse::<i64>().unwrap_or(1);
                                                }
                                                edit_loots.set(loots);
                                            },
                                        }
                                        Input {
                                            placeholder: "Classes (Standard, Warrior…)",
                                            r#type: "text",
                                            value: "{loot.classes}",
                                            oninput: move |e: FormEvent| {
                                                let mut loots = edit_loots();
                                                if let Some(item) = loots.get_mut(idx) {
                                                    item.classes = e.value();
                                                }
                                                edit_loots.set(loots);
                                            },
                                        }
                                        Button {
                                            variant: ButtonVariant::Destructive,
                                            onclick: move |_| {
                                                let mut loots = edit_loots();
                                                loots.remove(idx_rm);
                                                edit_loots.set(loots);
                                            },
                                            "✕"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let mut loots = edit_loots();
                            loots
                                .push(ScenarioLootItem {
                                    name: String::new(),
                                    kind: "Equipment".to_owned(),
                                    rank: "Common".to_owned(),
                                    level: 1,
                                    classes: "Standard".to_owned(),
                                });
                            edit_loots.set(loots);
                        },
                        "＋ Add Loot"
                    }

                    div { style: "display:flex;gap:8px;margin-top:8px;",
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                let u = selected_universe();
                                let fs = edit_file_stem();
                                let level = edit_level().trim().parse::<u64>().unwrap_or(1);
                                let detail = ScenarioDetail {
                                    name: edit_name(),
                                    description: edit_description(),
                                    level,
                                    boss_patterns_text: edit_bosses(),
                                    loots: edit_loots(),
                                };
                                spawn(async move {
                                    if fs.trim().is_empty() {
                                        feedback.set("❌ File stem cannot be empty.".to_owned());
                                        return;
                                    }
                                    if detail.name.trim().is_empty() {
                                        feedback.set("❌ Name cannot be empty.".to_owned());
                                        return;
                                    }
                                    match save_scenario_detail(u.clone(), fs, detail).await {
                                        Ok(()) => {
                                            feedback.set("✅ Saved.".to_owned());
                                            edit_mode.set(ScenarioEditMode::None);
                                            universes_resource.restart();
                                            if let Ok(mut s) = admin_list_scenarios().await {
                                                s.retain(|sc| sc.universe == u);
                                                s.sort_by_key(|sc| sc.level);
                                                scenarios.set(s);
                                            }
                                        }
                                        Err(e) => feedback.set(format!("❌ {e}")),
                                    }
                                });
                            },
                            "💾 Save"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| {
                                edit_mode.set(ScenarioEditMode::None);
                                feedback.set(String::new());
                            },
                            "Cancel"
                        }
                    }
                }
            }

            if !feedback().is_empty() {
                p { class: if feedback().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                    "{feedback}"
                }
            }
        }
    }
}

// ─── Characters Tab ───────────────────────────────────────────────────────────

#[component]
fn AdminCharactersTab() -> Element {
    let mut characters: Signal<Vec<AdminCharacterInfo>> = use_signal(Vec::new);
    let mut boss_characters: Signal<Vec<AdminCharacterInfo>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut selected_universe = use_signal(String::new);
    let mut show_bosses = use_signal(|| false);
    let universes_resource = use_resource(list_universes_server);

    // Character edit state
    let mut edit_char_name: Signal<Option<String>> = use_signal(|| None);
    let mut char_json = use_signal(String::new);
    let mut char_edit_form_mode = use_signal(|| false);
    let mut char_feedback = use_signal(String::new);

    // Character form signals
    let mut form_name = use_signal(String::new);
    let mut form_short_name = use_signal(String::new);
    let mut form_class = use_signal(String::new);
    let mut form_level = use_signal(|| "1".to_string());
    let mut form_photo = use_signal(String::new);
    let mut form_char_type = use_signal(|| "Hero".to_string());
    let mut form_rank = use_signal(|| "Common".to_string());
    let mut form_color = use_signal(String::new);
    let mut form_description = use_signal(String::new);
    let mut form_max_actions = use_signal(|| "1".to_string());
    let mut form_energies: Signal<Vec<String>> = use_signal(Vec::new);
    let mut form_is_blocking_atk = use_signal(|| false);
    let mut form_stats: Signal<Vec<StatEntry>> = use_signal(Vec::new);

    // Attack management state
    let mut attacks_char: Signal<Option<String>> = use_signal(|| None);
    let mut attacks_list: Signal<Vec<String>> = use_signal(Vec::new);
    let mut edit_attack_name: Signal<Option<String>> = use_signal(|| None);
    let mut attack_edit_form_mode = use_signal(|| false);
    let mut new_attack_name = use_signal(String::new);
    let mut attack_json = use_signal(String::new);
    let mut attack_feedback = use_signal(String::new);

    // Attack form signals
    let mut atk_nom = use_signal(String::new);
    let mut atk_niveau = use_signal(|| "1".to_string());
    let mut atk_description = use_signal(String::new);
    let mut atk_cible = use_signal(|| "Enemy".to_string());
    let mut atk_portee = use_signal(|| "Individual".to_string());
    let mut atk_forme = use_signal(|| "Standard".to_string());
    let mut atk_cout_mana = use_signal(|| "0".to_string());
    let mut atk_cout_rage = use_signal(|| "0".to_string());
    let mut atk_cout_vigueur = use_signal(|| "0".to_string());
    let mut atk_duree = use_signal(|| "1".to_string());
    let mut atk_aggro = use_signal(|| "0".to_string());
    let mut atk_photo = use_signal(String::new);
    let mut atk_effet = use_signal(|| "[]".to_string());

    use_effect(move || {
        spawn(async move {
            match admin_list_characters().await {
                Ok(c) => characters.set(c),
                Err(e) => tracing::error!("admin_list_characters: {e}"),
            }
            match admin_list_bosses().await {
                Ok(b) => {
                    boss_characters.set(b);
                    loading.set(false);
                }
                Err(e) => {
                    tracing::error!("admin_list_bosses: {e}");
                    loading.set(false);
                }
            }
        });
    });

    let universes = universes_resource
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    let displayed: Vec<AdminCharacterInfo> = {
        let source = if show_bosses() {
            boss_characters()
        } else {
            characters()
        };
        if selected_universe().is_empty() {
            source
        } else {
            source
                .into_iter()
                .filter(|c| c.universe == selected_universe())
                .collect()
        }
    };

    let kind_label = if show_bosses() {
        "👹 Bosses"
    } else {
        "🧙 Heroes"
    };

    rsx! {
        // Universe filter
        div { class: "admin-card",
            p { class: "admin-section-title", "🌐 Filter by Universe" }
            select {
                class: "admin-select",
                value: "{selected_universe}",
                onchange: move |e| {
                    selected_universe.set(e.value());
                    edit_char_name.set(None);
                    attacks_char.set(None);
                },
                option { value: "", "— all universes —" }
                for u in &universes {
                    option { value: "{u}", "{u}" }
                }
            }
        }

        div { class: "admin-full-card",
            div { style: "display:flex;align-items:center;justify-content:space-between;margin-bottom:12px;",
                p { class: "admin-section-title", style: "margin:0;", "{kind_label}" }
                div { style: "display:flex;gap:8px;",
                    Button {
                        variant: if !show_bosses() { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                        onclick: move |_| {
                            show_bosses.set(false);
                            edit_char_name.set(None);
                            attacks_char.set(None);
                        },
                        "🧙 Heroes"
                    }
                    Button {
                        variant: if show_bosses() { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                        onclick: move |_| {
                            show_bosses.set(true);
                            edit_char_name.set(None);
                            attacks_char.set(None);
                        },
                        "👹 Bosses"
                    }
                }
            }

            if loading() {
                p { style: "color:var(--rpg-text-muted);", "Loading…" }
            } else if displayed.is_empty() {
                p { style: "color:var(--rpg-text-muted);", "No {kind_label} found." }
            } else {
                div { class: "admin-char-grid",
                    for c in displayed {
                        {
                            let name_form = c.db_full_name.clone();
                            let name_edit = c.db_full_name.clone();
                            let name_atk = c.db_full_name.clone();
                            let universe_form = c.universe.clone();
                            let universe_edit = c.universe.clone();
                            rsx! {
                                div { class: "admin-char-card",
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
                                                if !c.universe.is_empty() {
                                                    span { class: "admin-char-universe", "🌐 {c.universe}" }
                                                }
                                            }
                                        }
                                    }
                                    if !c.description.is_empty() {
                                        p { class: "admin-char-desc", "{c.description}" }
                                    }
                                    div { class: "admin-char-stats",
                                        {
                                            let mut sorted_stats: Vec<(String, (u64, u64))> = c.stats.into_iter().collect();
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
                                    div { style: "display:flex;gap:6px;margin-top:8px;flex-wrap:wrap;",
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            onclick: move |_| {
                                                let n = name_form.clone();
                                                let u = universe_form.clone();
                                                char_feedback.set(String::new());
                                                spawn(async move {
                                                    match admin_get_character_form(u, n.clone()).await {
                                                        Ok(form) => {
                                                            form_name.set(form.name);
                                                            form_short_name.set(form.short_name);
                                                            form_class.set(form.class);
                                                            form_level.set(form.level.to_string());
                                                            form_photo.set(form.photo);
                                                            form_char_type.set(form.char_type);
                                                            form_rank.set(form.rank);
                                                            form_color.set(form.color);
                                                            form_description.set(form.description);
                                                            form_max_actions.set(form.max_actions.to_string());
                                                            form_energies.set(form.energies);
                                                            form_is_blocking_atk.set(form.is_blocking_atk);
                                                            form_stats.set(form.stats);
                                                            char_edit_form_mode.set(true);
                                                            edit_char_name.set(Some(n));
                                                            attacks_char.set(None);
                                                        }
                                                        Err(e) => char_feedback.set(format!("❌ {e}")),
                                                    }
                                                });
                                            },
                                            "📝 Form"
                                        }
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            onclick: move |_| {
                                                let n = name_edit.clone();
                                                let u = universe_edit.clone();
                                                char_feedback.set(String::new());
                                                spawn(async move {
                                                    match admin_get_character_json(u, n.clone()).await {
                                                        Ok(json) => {
                                                            char_json.set(json);
                                                            char_edit_form_mode.set(false);
                                                            edit_char_name.set(Some(n));
                                                            attacks_char.set(None);
                                                        }
                                                        Err(e) => char_feedback.set(format!("❌ {e}")),
                                                    }
                                                });
                                            },
                                            "✏️ JSON"
                                        }
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            onclick: move |_| {
                                                let n = name_atk.clone();
                                                attack_feedback.set(String::new());
                                                edit_attack_name.set(None);
                                                attack_edit_form_mode.set(false);
                                                spawn(async move {
                                                    match admin_list_attacks(n.clone()).await {
                                                        Ok(list) => {
                                                            attacks_list.set(list);
                                                            attacks_char.set(Some(n));
                                                            edit_char_name.set(None);
                                                        }
                                                        Err(e) => attack_feedback.set(format!("❌ {e}")),
                                                    }
                                                });
                                            },
                                            "⚔️ Attacks"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Character editor ──
        if let Some(cname) = edit_char_name() {
            {
                let cname_form_switch = cname.clone();
                let cname_form_save = cname.clone();
                let cname_json_switch = cname.clone();
                let cname_json_save = cname.clone();
                rsx! {
                    div { class: "admin-full-card",
                        if char_edit_form_mode() {
                            div { style: "display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;",
                                p { class: "admin-section-title", style: "margin:0;", "📝 Form: {cname}" }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        let n = cname_form_switch.clone();
                                        let u = selected_universe();
                                        spawn(async move {
                                            if let Ok(json) = admin_get_character_json(u, n).await {
                                                char_json.set(json);
                                            }
                                        });
                                        char_edit_form_mode.set(false);
                                    },
                                    "✏️ JSON mode"
                                }
                            }
                            div { class: "admin-form-grid",
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-name",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Name"
                                    }
                                    Input {
                                        placeholder: "Character name",
                                        r#type: "text",
                                        value: "{form_name}",
                                        oninput: move |e: FormEvent| form_name.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-short-name",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Short name"
                                    }
                                    Input {
                                        placeholder: "Short name",
                                        r#type: "text",
                                        value: "{form_short_name}",
                                        oninput: move |e: FormEvent| form_short_name.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field", // Description
                                    Label {
                                        html_for: "char-class",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Class" // Energies & toggles
                                    }
                                    select {
                                        class: "admin-select",
                                        value: "{form_class}",
                                        onchange: move |e| form_class.set(e.value()),
                                        option {
                                            value: "Standard",
                                            selected: form_class() == "Standard",
                                            "Standard"
                                        }
                                        option { value: "Warrior", selected: form_class() == "Warrior", "Warrior" }
                                        option { value: "Mage", selected: form_class() == "Mage", "Mage" }
                                        option { value: "Healer", selected: form_class() == "Healer", "Healer" }
                                        option {
                                            value: "Berserker",
                                            selected: form_class() == "Berserker",
                                            "Berserker"
                                        }
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-level",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Level"
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{form_level}",
                                        oninput: move |e: FormEvent| form_level.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-rank",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Rank"
                                    }
                                    select {
                                        class: "admin-select",
                                        value: "{form_rank}",
                                        onchange: move |e| form_rank.set(e.value()),
                                        option { value: "Common", selected: form_rank() == "Common", "Common" }
                                        option {
                                            value: "Intermediate",
                                            selected: form_rank() == "Intermediate",
                                            "Intermediate"
                                        }
                                        option {
                                            value: "Advanced",
                                            selected: form_rank() == "Advanced",
                                            "Advanced"
                                        }
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-type",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Type"
                                    }
                                    select {
                                        class: "admin-select",
                                        value: "{form_char_type}",
                                        onchange: move |e| form_char_type.set(e.value()),
                                        option { value: "Hero", selected: form_char_type() == "Hero", "Hero" }
                                        option { value: "Boss", selected: form_char_type() == "Boss", "Boss" }
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-photo",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Photo (without extension)"
                                    }
                                    Input {
                                        placeholder: "e.g. Thalia",
                                        r#type: "text",
                                        value: "{form_photo}",
                                        oninput: move |e: FormEvent| form_photo.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-color",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Color"
                                    }
                                    Input {
                                        placeholder: "e.g. green",
                                        r#type: "text",
                                        value: "{form_color}",
                                        oninput: move |e: FormEvent| form_color.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-max-actions",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Max actions / round"
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{form_max_actions}",
                                        oninput: move |e: FormEvent| form_max_actions.set(e.value()),
                                    }
                                }
                            }
                            // Description
                            Label {
                                html_for: "char-description",
                                color: "var(--rpg-text-muted)",
                                font_size: "0.82rem",
                                "Description"
                            }
                            textarea {
                                class: "admin-json-textarea",
                                rows: "3",
                                placeholder: "Character description…",
                                value: "{form_description}",
                                oninput: move |e: FormEvent| form_description.set(e.value()),
                            }
                            // Energies & toggles
                            div { style: "display:flex;flex-wrap:wrap;gap:16px;align-items:center;margin:10px 0;",
                                div { style: "display:flex;flex-direction:column;gap:4px;",
                                    p { style: "font-size:0.82rem;color:var(--rpg-text-muted);margin:0 0 4px;",
                                        "Energies"
                                    }
                                    div { style: "display:flex;gap:10px;flex-wrap:wrap;",
                                        for energy in ["Mana", "Rage", "Vigor"] {
                                            {
                                                let e = energy;
                                                let has = form_energies().contains(&e.to_owned());
                                                rsx! {
                                                    label { style: "display:flex;align-items:center;gap:4px;cursor:pointer;font-size:0.9rem;",
                                                        input {
                                                            r#type: "checkbox",
                                                            checked: has,
                                                            onchange: move |_| {
                                                                let mut energies = form_energies();
                                                                if energies.contains(&e.to_owned()) {
                                                                    energies.retain(|x| x != e);
                                                                } else {
                                                                    energies.push(e.to_owned());
                                                                }
                                                                form_energies.set(energies);
                                                            },
                                                        }
                                                        "{e}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                label { style: "display:flex;align-items:center;gap:6px;cursor:pointer;font-size:0.9rem;",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_is_blocking_atk(),
                                        onchange: move |_| form_is_blocking_atk.set(!form_is_blocking_atk()),
                                    }
                                    "Is blocking attack"
                                }
                            }
                            // Stats table
                            if !form_stats().is_empty() {
                                p { style: "font-weight:600;margin:12px 0 6px;color:var(--rpg-text-muted);font-size:0.82rem;",
                                    "Stats"
                                }
                                div { class: "admin-stats-table",
                                    div { class: "admin-stats-header",
                                        span { class: "ast-col-name", "Stat" }
                                        span { class: "ast-col-val", "Current" }
                                        span { class: "ast-col-sep", "" }
                                        span { class: "ast-col-val", "Max" }
                                    }
                                    for (idx, stat) in form_stats().iter().enumerate() {
                                        {
                                            let sname = stat.stat_name.clone();
                                            let idx_cur = idx;
                                            let idx_max = idx;
                                            rsx! {
                                                div { class: "admin-stats-row",
                                                    span { class: "ast-col-name", "{sname}" }
                                                    input {
                                                        class: "ast-input",
                                                        r#type: "number",
                                                        value: "{stat.current}",
                                                        oninput: move |e: FormEvent| {
                                                            let mut stats = form_stats();
                                                            if let Some(s) = stats.get_mut(idx_cur) {
                                                                s.current = e.value().trim().parse::<i64>().unwrap_or(0);
                                                            }
                                                            form_stats.set(stats);
                                                        },
                                                    }
                                                    span { class: "ast-col-sep", "/" }
                                                    input {
                                                        class: "ast-input",
                                                        r#type: "number",
                                                        value: "{stat.max}",
                                                        oninput: move |e: FormEvent| {
                                                            let mut stats = form_stats();
                                                            if let Some(s) = stats.get_mut(idx_max) {
                                                                s.max = e.value().trim().parse::<i64>().unwrap_or(0);
                                                            }
                                                            form_stats.set(stats);
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { style: "display:flex;gap:8px;margin-top:12px;",
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| {
                                        let name = cname_form_save.clone();
                                        let u = selected_universe();
                                        let form = CharacterFormData {
                                            name: form_name(),
                                            short_name: form_short_name(),
                                            class: form_class(),
                                            level: form_level().trim().parse::<u64>().unwrap_or(1),
                                            photo: form_photo(),
                                            char_type: form_char_type(),
                                            rank: form_rank(),
                                            color: form_color(),
                                            description: form_description(),
                                            max_actions: form_max_actions().trim().parse::<i64>().unwrap_or(1),
                                            energies: form_energies(),
                                            is_blocking_atk: form_is_blocking_atk(),
                                            stats: form_stats(),
                                        };
                                        spawn(async move {
                                            match admin_save_character_form(u.clone(), name.clone(), form).await {
                                                Ok(()) => {
                                                    char_feedback.set("✅ Saved.".to_owned());
                                                    edit_char_name.set(None);
                                                    if let Ok(c) = admin_list_characters().await {
                                                        characters.set(c);
                                                    }
                                                    if let Ok(b) = admin_list_bosses().await {
                                                        boss_characters.set(b);
                                                    }
                                                }
                                                Err(e) => char_feedback.set(format!("❌ {e}")),
                                            }
                                        });
                                    },
                                    "💾 Save"
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        edit_char_name.set(None);
                                        char_feedback.set(String::new());
                                    },
                                    "Cancel"
                                }
                            }
                        } else {
                            div { style: "display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;",
                                p { class: "admin-section-title", style: "margin:0;", "✏️ JSON: {cname}" }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        let n = cname_json_switch.clone();
                                        let u = selected_universe();
                                        spawn(async move {
                                            if let Ok(form) = admin_get_character_form(u, n.clone()).await {
                                                form_name.set(form.name);
                                                form_short_name.set(form.short_name);
                                                form_class.set(form.class);
                                                form_level.set(form.level.to_string());
                                                form_photo.set(form.photo);
                                                form_char_type.set(form.char_type);
                                                form_rank.set(form.rank);
                                                form_color.set(form.color);
                                                form_description.set(form.description);
                                                form_max_actions.set(form.max_actions.to_string());
                                                form_energies.set(form.energies);
                                                form_is_blocking_atk.set(form.is_blocking_atk);
                                                form_stats.set(form.stats);
                                                char_edit_form_mode.set(true);
                                            }
                                        });
                                    },
                                    "📝 Form mode"
                                }
                            }
                            textarea {
                                class: "admin-json-textarea",
                                rows: "24",
                                value: "{char_json}",
                                oninput: move |e: FormEvent| char_json.set(e.value()),
                            }
                            div { style: "display:flex;gap:8px;margin-top:8px;",
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| {
                                        let name = cname_json_save.clone();
                                        let u = selected_universe();
                                        let json = char_json();
                                        spawn(async move {
                                            match admin_save_character_json(u, name.clone(), json).await {
                                                Ok(()) => {
                                                    char_feedback.set("✅ Saved.".to_owned());
                                                    edit_char_name.set(None);
                                                    if let Ok(c) = admin_list_characters().await {
                                                        characters.set(c);
                                                    }
                                                    if let Ok(b) = admin_list_bosses().await {
                                                        boss_characters.set(b);
                                                    }
                                                }
                                                Err(e) => char_feedback.set(format!("❌ {e}")),
                                            }
                                        });
                                    },
                                    "💾 Save"
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        edit_char_name.set(None);
                                        char_feedback.set(String::new());
                                    },
                                    "Cancel"
                                }
                            }
                        }
                        if !char_feedback().is_empty() {
                            p { class: if char_feedback().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                                "{char_feedback}"
                            }
                        }
                    }
                }
            } // close Rust block
        } // close if let Some(cname)
        if let Some(achar) = attacks_char() {
            div { class: "admin-full-card",
                p { class: "admin-section-title", "⚔️ Attacks — {achar}" }

                if attacks_list().is_empty() {
                    p { style: "color:var(--rpg-text-muted);", "No attacks found." }
                } else {
                    div { style: "display:flex;flex-wrap:wrap;gap:6px;margin-bottom:12px;",
                        for atk in attacks_list() {
                            {
                                let atk_name = atk.clone();
                                let achar2 = achar.clone();
                                rsx! {
                                    Button {
                                        variant: if edit_attack_name() == Some(atk_name.clone()) { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                        onclick: move |_| {
                                            let n = atk_name.clone();
                                            let c = achar2.clone();
                                            attack_feedback.set(String::new());
                                            attack_edit_form_mode.set(false);
                                            spawn(async move {
                                                match admin_get_attack_json(c, n.clone()).await {
                                                    Ok(json) => {
                                                        attack_json.set(json);
                                                        edit_attack_name.set(Some(n));
                                                    }
                                                    Err(e) => attack_feedback.set(format!("❌ {e}")),
                                                }
                                            });
                                        },
                                        "{atk}"
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(aname) = edit_attack_name() {
                    {
                        let achar_a = achar.clone();
                        let achar_b = achar.clone();
                        let achar_c = achar.clone();
                        let _achar_d = achar.clone();
                        let aname_a = aname.clone();
                        let aname_b = aname.clone();
                        let aname_c = aname.clone();
                        let _aname_d = aname.clone();
                        rsx! {
                            if attack_edit_form_mode() {
                                div { style: "display:flex;align-items:center;justify-content:space-between;margin:8px 0 4px;",
                                    p { style: "font-weight:600;", "📝 {aname}" }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        onclick: move |_| {
                                            let c = achar_a.clone();
                                            let n = aname_a.clone();
                                            spawn(async move {
                                                if let Ok(json) = admin_get_attack_json(c, n).await {
                                                    attack_json.set(json);
                                                }
                                            });
                                            attack_edit_form_mode.set(false);
                                        },
                                        "✏️ JSON mode"
                                    }
                                }
                                div { class: "admin-form-grid",
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-nom",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Nom"
                                        }
                                        Input {
                                            r#type: "text",
                                            value: "{atk_nom}",
                                            oninput: move |e: FormEvent| atk_nom.set(e.value()),
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-niveau",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Niveau"
                                        }
                                        Input {
                                            r#type: "number",
                                            value: "{atk_niveau}",
                                            oninput: move |e: FormEvent| atk_niveau.set(e.value()),
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-cible",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Cible"
                                        }
                                        select {
                                            class: "admin-select",
                                            value: "{atk_cible}",
                                            onchange: move |e| atk_cible.set(e.value()),
                                            option { value: "Enemy", "Enemy" }
                                            option { value: "Ally", "Ally" }
                                            option { value: "Self", "Self" }
                                            option { value: "Zone", "Zone" }
                                            option { value: "All", "All" }
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-portee",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Portée"
                                        }
                                        select {
                                            class: "admin-select",
                                            value: "{atk_portee}",
                                            onchange: move |e| atk_portee.set(e.value()),
                                            option { value: "Individual", "Individual" }
                                            option { value: "Area", "Area" }
                                            option { value: "All", "All" }
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-forme",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Forme"
                                        }
                                        select {
                                            class: "admin-select",
                                            value: "{atk_forme}",
                                            onchange: move |e| atk_forme.set(e.value()),
                                            option { value: "Standard", "Standard" }
                                            option { value: "Magic", "Magic" }
                                            option { value: "Healing", "Healing" }
                                            option { value: "Support", "Support" }
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-photo",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Photo"
                                        }
                                        Input {
                                            r#type: "text",
                                            placeholder: "e.g. Fireball.png",
                                            value: "{atk_photo}",
                                            oninput: move |e: FormEvent| atk_photo.set(e.value()),
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-mana",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Coût Mana"
                                        }
                                        Input {
                                            r#type: "number",
                                            value: "{atk_cout_mana}",
                                            oninput: move |e: FormEvent| atk_cout_mana.set(e.value()),
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-rage",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Coût Rage"
                                        }
                                        Input {
                                            r#type: "number",
                                            value: "{atk_cout_rage}",
                                            oninput: move |e: FormEvent| atk_cout_rage.set(e.value()),
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-vigueur",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Coût Vigueur"
                                        }
                                        Input {
                                            r#type: "number",
                                            value: "{atk_cout_vigueur}",
                                            oninput: move |e: FormEvent| atk_cout_vigueur.set(e.value()),
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-duree",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Durée"
                                        }
                                        Input {
                                            r#type: "number",
                                            value: "{atk_duree}",
                                            oninput: move |e: FormEvent| atk_duree.set(e.value()),
                                        }
                                    }
                                    div { class: "admin-form-field",
                                        Label {
                                            html_for: "atk-aggro",
                                            color: "var(--rpg-text-muted)",
                                            font_size: "0.82rem",
                                            "Aggro"
                                        }
                                        Input {
                                            r#type: "number",
                                            value: "{atk_aggro}",
                                            oninput: move |e: FormEvent| atk_aggro.set(e.value()),
                                        }
                                    }
                                }
                                Label {
                                    html_for: "atk-description",
                                    color: "var(--rpg-text-muted)",
                                    font_size: "0.82rem",
                                    "Description"
                                }
                                textarea {
                                    class: "admin-json-textarea",
                                    rows: "3",
                                    value: "{atk_description}",
                                    oninput: move |e: FormEvent| atk_description.set(e.value()),
                                }
                                Label {
                                    html_for: "atk-effet",
                                    color: "var(--rpg-text-muted)",
                                    font_size: "0.82rem",
                                    "Effet (JSON array)"
                                }
                                textarea {
                                    class: "admin-json-textarea",
                                    rows: "8",
                                    value: "{atk_effet}",
                                    oninput: move |e: FormEvent| atk_effet.set(e.value()),
                                }
                                div { style: "display:flex;gap:8px;margin-top:8px;",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| {
                                            let c = achar_b.clone();
                                            let n = aname_b.clone();
                                            let form = AttackFormData {
                                                nom: atk_nom(),
                                                niveau: atk_niveau().trim().parse::<i64>().unwrap_or(1),
                                                description: atk_description(),
                                                cible: atk_cible(),
                                                portee: atk_portee(),
                                                forme: atk_forme(),
                                                cout_mana: atk_cout_mana().trim().parse::<i64>().unwrap_or(0),
                                                cout_rage: atk_cout_rage().trim().parse::<i64>().unwrap_or(0),
                                                cout_vigueur: atk_cout_vigueur().trim().parse::<i64>().unwrap_or(0),
                                                duree: atk_duree().trim().parse::<i64>().unwrap_or(1),
                                                aggro: atk_aggro().trim().parse::<i64>().unwrap_or(0),
                                                photo: atk_photo(),
                                                effet_json: atk_effet(),
                                            };
                                            spawn(async move {
                                                match admin_save_attack_form(c, n, form).await {
                                                    Ok(()) => attack_feedback.set("✅ Saved.".to_owned()),
                                                    Err(e) => attack_feedback.set(format!("❌ {e}")),
                                                }
                                            });
                                        },
                                        "💾 Save"
                                    }
                                    Button {
                                        variant: ButtonVariant::Destructive,
                                        onclick: move |_| {
                                            let c = achar_c.clone();
                                            let n = aname_c.clone();
                                            spawn(async move {
                                                match admin_delete_attack(c.clone(), n).await {
                                                    Ok(()) => {
                                                        attack_feedback.set("✅ Deleted.".to_owned());
                                                        edit_attack_name.set(None);
                                                        if let Ok(list) = admin_list_attacks(c).await {
                                                            attacks_list.set(list);
                                                        }
                                                    }
                                                    Err(e) => attack_feedback.set(format!("❌ {e}")),
                                                }
                                            });
                                        },
                                        "🗑️ Delete"
                                    }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        onclick: move |_| {
                                            edit_attack_name.set(None);
                                            attack_feedback.set(String::new());
                                        },
                                        "Cancel"
                                    }
                                }
                            } else {
                                div { style: "display:flex;align-items:center;justify-content:space-between;margin:8px 0 4px;",
                                    p { style: "font-weight:600;", "✏️ {aname}" }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        onclick: move |_| {
                                            let c = achar_a.clone();
                                            let n = aname_a.clone();
                                            spawn(async move {
                                                match admin_get_attack_form(c, n.clone()).await {
                                                    Ok(form) => {
                                                        atk_nom.set(form.nom);
                                                        atk_niveau.set(form.niveau.to_string());
                                                        atk_description.set(form.description);
                                                        atk_cible.set(form.cible);
                                                        atk_portee.set(form.portee);
                                                        atk_forme.set(form.forme);
                                                        atk_cout_mana.set(form.cout_mana.to_string());
                                                        atk_cout_rage.set(form.cout_rage.to_string());
                                                        atk_cout_vigueur.set(form.cout_vigueur.to_string());
                                                        atk_duree.set(form.duree.to_string());
                                                        atk_aggro.set(form.aggro.to_string());
                                                        atk_photo.set(form.photo);
                                                        atk_effet.set(form.effet_json);
                                                        attack_edit_form_mode.set(true);
                                                    }
                                                    Err(e) => attack_feedback.set(format!("❌ {e}")),
                                                }
                                            });
                                        },
                                        "📝 Form mode"
                                    }
                                }
                                textarea {
                                    class: "admin-json-textarea",
                                    rows: "16",
                                    value: "{attack_json}",
                                    oninput: move |e: FormEvent| attack_json.set(e.value()),
                                }
                                div { style: "display:flex;gap:8px;margin-top:6px;",
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| {
                                            let c = achar_b.clone();
                                            let n = aname_b.clone();
                                            let json = attack_json();
                                            spawn(async move {
                                                match admin_save_attack_json(c.clone(), n, json).await {
                                                    Ok(()) => attack_feedback.set("✅ Saved.".to_owned()),
                                                    Err(e) => attack_feedback.set(format!("❌ {e}")),
                                                }
                                            });
                                        },
                                        "💾 Save"
                                    }
                                    Button {
                                        variant: ButtonVariant::Destructive,
                                        onclick: move |_| {
                                            let c = achar_c.clone();
                                            let n = aname_c.clone();
                                            spawn(async move {
                                                match admin_delete_attack(c.clone(), n).await {
                                                    Ok(()) => {
                                                        attack_feedback.set("✅ Deleted.".to_owned());
                                                        edit_attack_name.set(None);
                                                        if let Ok(list) = admin_list_attacks(c).await {
                                                            attacks_list.set(list);
                                                        }
                                                    }
                                                    Err(e) => attack_feedback.set(format!("❌ {e}")),
                                                }
                                            });
                                        },
                                        "🗑️ Delete"
                                    }
                                    Button {
                                        variant: ButtonVariant::Secondary,
                                        onclick: move |_| {
                                            edit_attack_name.set(None);
                                            attack_feedback.set(String::new());
                                        },
                                        "Cancel"
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "margin-top:16px;border-top:1px solid var(--rpg-border);padding-top:12px;",
                    p { style: "font-weight:600;margin-bottom:6px;", "➕ New Attack" }
                    div { style: "display:flex;gap:8px;align-items:center;",
                        Input {
                            placeholder: "Attack file name (e.g. Fireball)",
                            r#type: "text",
                            value: "{new_attack_name}",
                            oninput: move |e: FormEvent| new_attack_name.set(e.value()),
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                let n = new_attack_name().trim().to_owned();
                                let c = attacks_char().unwrap_or_default();
                                if n.is_empty() {
                                    return;
                                }
                                let template = serde_json::json!(
                                    { "Nom" : n, "Niveau" : 1, "Description" : "", "Cible" : "Enemy", "Portée" :
                                    "Individual", "Forme" : "Standard", "Coût de mana" : 0, "Coût de rage" : 0,
                                    "Coût de vigueur" : 0, "Durée" : 1, "Aggro" : 0, "Photo" :
                                    format!("{}.png", n), "Effet" : [] }
                                );
                                let json = serde_json::to_string_pretty(&template).unwrap_or_default();
                                spawn(async move {
                                    match admin_save_attack_json(c.clone(), n.clone(), json).await {
                                        Ok(()) => {
                                            new_attack_name.set(String::new());
                                            attack_feedback.set("✅ Created.".to_owned());
                                            if let Ok(list) = admin_list_attacks(c).await {
                                                attacks_list.set(list);
                                            }
                                        }
                                        Err(e) => attack_feedback.set(format!("❌ {e}")),
                                    }
                                });
                            },
                            "Create"
                        }
                    }
                }

                if !attack_feedback().is_empty() {
                    p { class: if attack_feedback().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                        "{attack_feedback}"
                    }
                }
            }
        }

        if !char_feedback().is_empty() && edit_char_name().is_none() {
            p { class: if char_feedback().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                "{char_feedback}"
            }
        }
    }
}

// ─── Equipment Tab ────────────────────────────────────────────────────────────

#[component]
fn AdminEquipmentTab() -> Element {
    let mut eq_types: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_type = use_signal(String::new);
    let mut eq_categories: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_category = use_signal(String::new);
    let mut eq_items: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_item: Signal<Option<String>> = use_signal(|| None);
    let mut eq_json = use_signal(String::new);
    let mut feedback = use_signal(String::new);
    let mut confirm_delete_item: Signal<Option<String>> = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            if let Ok(types) = admin_list_equipment_types().await {
                eq_types.set(types);
            }
        });
    });

    rsx! {
        div { class: "admin-card",
            p { class: "admin-section-title", "🔧 Equipment Browser" }
            div { style: "display:flex;gap:12px;flex-wrap:wrap;",
                select {
                    class: "admin-select",
                    value: "{selected_type}",
                    onchange: move |e| {
                        let t = e.value();
                        selected_type.set(t.clone());
                        selected_category.set(String::new());
                        eq_categories.set(Vec::new());
                        eq_items.set(Vec::new());
                        selected_item.set(None);
                        feedback.set(String::new());
                        if !t.is_empty() {
                            spawn(async move {
                                if let Ok(cats) = admin_list_equipment_categories(t).await {
                                    eq_categories.set(cats);
                                }
                            });
                        }
                    },
                    option { value: "", "— type —" }
                    for t in eq_types() {
                        option { value: "{t}", "{t}" }
                    }
                }
                if !eq_categories().is_empty() {
                    select {
                        class: "admin-select",
                        value: "{selected_category}",
                        onchange: move |e| {
                            let c = e.value();
                            let t = selected_type();
                            selected_category.set(c.clone());
                            eq_items.set(Vec::new());
                            selected_item.set(None);
                            feedback.set(String::new());
                            if !c.is_empty() {
                                spawn(async move {
                                    if let Ok(items) = admin_list_equipment_items(t, c).await {
                                        eq_items.set(items);
                                    }
                                });
                            }
                        },
                        option { value: "", "— category —" }
                        for c in eq_categories() {
                            option { value: "{c}", "{c}" }
                        }
                    }
                }
            }
        }

        if !eq_items().is_empty() {
            div { class: "admin-full-card",
                p { class: "admin-section-title", "📋 Items — {selected_category}" }
                div { style: "display:flex;flex-wrap:wrap;gap:6px;",
                    for item in eq_items() {
                        {
                            let item_btn = item.clone();
                            let item_del = item.clone();
                            let item_del2 = item.clone();
                            let t_btn = selected_type();
                            let c_btn = selected_category();
                            let t_del = t_btn.clone();
                            let c_del = c_btn.clone();
                            let _t_del2 = t_del.clone();
                            let _c_del2 = c_del.clone();
                            let is_confirm = confirm_delete_item() == Some(item_del.clone());
                            rsx! {
                                div { style: "display:flex;gap:3px;",
                                    Button {
                                        variant: if selected_item() == Some(item_btn.clone()) { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                        onclick: move |_| {
                                            let n = item_btn.clone();
                                            let t = t_btn.clone();
                                            let c = c_btn.clone();
                                            feedback.set(String::new());
                                            spawn(async move {
                                                match admin_get_equipment_json(t, c, n.clone()).await {
                                                    Ok(json) => {
                                                        eq_json.set(json);
                                                        selected_item.set(Some(n));
                                                    }
                                                    Err(e) => feedback.set(format!("❌ {e}")),
                                                }
                                            });
                                        },
                                        "{item}"
                                    }
                                    if is_confirm {
                                        Button {
                                            variant: ButtonVariant::Destructive,
                                            onclick: move |_| {
                                                let t = t_del.clone();
                                                let c = c_del.clone();
                                                let n = item_del.clone();
                                                spawn(async move {
                                                    match admin_delete_equipment(t.clone(), c.clone(), n).await {
                                                        Ok(()) => {
                                                            feedback.set("✅ Deleted.".to_owned());
                                                            confirm_delete_item.set(None);
                                                            selected_item.set(None);
                                                            if let Ok(items) = admin_list_equipment_items(t, c).await {
                                                                eq_items.set(items);
                                                            }
                                                        }
                                                        Err(e) => feedback.set(format!("❌ {e}")),
                                                    }
                                                });
                                            },
                                            "⚠️"
                                        }
                                        Button {
                                            variant: ButtonVariant::Secondary,
                                            onclick: move |_| confirm_delete_item.set(None),
                                            "✗"
                                        }
                                    } else {
                                        Button {
                                            variant: ButtonVariant::Destructive,
                                            onclick: move |_| confirm_delete_item.set(Some(item_del2.clone())),
                                            "✕"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(item_name) = selected_item() {
            div { class: "admin-full-card",
                p { class: "admin-section-title", "✏️ Edit: {item_name}" }
                textarea {
                    class: "admin-json-textarea",
                    rows: "26",
                    value: "{eq_json}",
                    oninput: move |e: FormEvent| eq_json.set(e.value()),
                }
                div { style: "display:flex;gap:8px;margin-top:8px;",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {
                            let t = selected_type();
                            let c = selected_category();
                            let n = item_name.clone();
                            let json = eq_json();
                            spawn(async move {
                                match admin_save_equipment_json(t, c, n, json).await {
                                    Ok(()) => feedback.set("✅ Saved.".to_owned()),
                                    Err(e) => feedback.set(format!("❌ {e}")),
                                }
                            });
                        },
                        "💾 Save"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            selected_item.set(None);
                            eq_json.set(String::new());
                            feedback.set(String::new());
                        },
                        "Cancel"
                    }
                }
            }
        }

        if !feedback().is_empty() {
            p { class: if feedback().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                "{feedback}"
            }
        }
    }
}

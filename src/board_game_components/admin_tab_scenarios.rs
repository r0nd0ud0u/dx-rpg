use dioxus::logger::tracing;
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::{
    auth_manager::server_fn::{
        AdminScenarioInfo, ScenarioDetail, ScenarioLootItem, admin_list_scenarios,
        delete_scenario_json, get_scenario_detail, list_universes_server, save_scenario_detail,
    },
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        label::Label,
    },
};

#[derive(Clone, PartialEq)]
enum ScenarioEditMode {
    None,
    Edit(String),
    New,
}

#[component]
pub fn AdminScenariosTab() -> Element {
    let mut scenarios: Signal<Vec<AdminScenarioInfo>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut selected_universe = use_signal(String::new);
    let mut universes_resource = use_resource(list_universes_server);
    let mut edit_mode = use_signal(|| ScenarioEditMode::None);
    let mut edit_file_stem = use_signal(String::new);
    let mut edit_name = use_signal(String::new);
    let mut edit_description = use_signal(String::new);
    let mut edit_level = use_signal(|| "1".to_owned());
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
            p { class: "admin-section-title", {t!("admin-scenarios-select-universe")} }
            select {
                class: "admin-select",
                value: "{selected_universe}",
                onchange: move |e| {
                    selected_universe.set(e.value());
                    edit_mode.set(ScenarioEditMode::None);
                    feedback.set(String::new());
                    confirm_delete.set(String::new());
                },
                option { value: "", {t!("admin-scenarios-choose-universe")} }
                for u in &universes {
                    option { value: "{u}", "{u}" }
                }
            }
        }

        if !selected_universe().is_empty() {
            div { class: "admin-full-card",
                p { class: "admin-section-title",
                    {t!("admin-scenarios-title", universe : selected_universe())}
                }

                if loading() {
                    p { style: "color:var(--rpg-text-muted);", {t!("common-loading")} }
                } else if scenarios().is_empty() {
                    p { style: "color:var(--rpg-text-muted);", {t!("admin-scenarios-empty")} }
                } else {
                    table { class: "admin-table",
                        thead {
                            tr {
                                th { class: "col-level", {t!("admin-scenarios-col-level")} }
                                th { class: "col-name", {t!("admin-equip-name-label")} }
                                th { class: "col-bosses", {t!("admin-scenarios-col-bosses")} }
                                th { class: "col-description",
                                    {t!("admin-scenarios-col-description")}
                                }
                                th { class: "col-file", {t!("admin-scenarios-col-file")} }
                                th { {t!("admin-scenarios-col-actions")} }
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
                                                                    Err(e) => feedback.set(t!("admin-error", error : e.to_string())),
                                                                }
                                                            });
                                                        },
                                                        {t!("admin-scenarios-edit")}
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
                                                                            feedback.set(t!("admin-deleted"));
                                                                            confirm_delete.set(String::new());
                                                                            edit_mode.set(ScenarioEditMode::None);
                                                                            if let Ok(mut s) = admin_list_scenarios().await {
                                                                                s.retain(|sc| sc.universe == u);
                                                                                s.sort_by_key(|sc| sc.level);
                                                                                scenarios.set(s);
                                                                            }
                                                                        }
                                                                        Err(e) => {
                                                                            feedback.set(t!("admin-error", error : e.to_string()));
                                                                            confirm_delete.set(String::new());
                                                                        }
                                                                    }
                                                                });
                                                            },
                                                            {t!("admin-scenarios-confirm-delete")}
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Secondary,
                                                            onclick: move |_| confirm_delete.set(String::new()),
                                                            {t!("common-cancel")}
                                                        }
                                                    } else {
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: {
                                                                let fs = file_stem.clone();
                                                                move |_| confirm_delete.set(fs.clone())
                                                            },
                                                            {t!("admin-scenarios-delete")}
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
                                edit_level.set("1".to_owned());
                                edit_bosses.set(String::new());
                                edit_loots.set(Vec::new());
                                edit_mode.set(ScenarioEditMode::New);
                                feedback.set(String::new());
                            },
                            {t!("admin-scenarios-add")}
                        }
                    }
                }
            }

            if edit_mode() != ScenarioEditMode::None {
                div { class: "admin-full-card",
                    p { class: "admin-section-title",
                        if edit_mode() == ScenarioEditMode::New {
                            {t!("admin-scenarios-new-title")}
                        } else {
                            {t!("admin-scenarios-edit-title")}
                        }
                    }

                    if edit_mode() == ScenarioEditMode::New {
                        Label {
                            html_for: "scenario-stem",
                            color: "var(--rpg-text-muted)",
                            font_size: "0.82rem",
                            {t!("admin-scenarios-file-stem-label")}
                        }
                        Input {
                            placeholder: t!("admin-scenarios-file-stem-placeholder"),
                            r#type: "text",
                            value: "{edit_file_stem}",
                            oninput: move |e: FormEvent| edit_file_stem.set(e.value()),
                        }
                    }

                    Label {
                        html_for: "scenario-name",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        {t!("admin-equip-name-label")}
                    }
                    Input {
                        placeholder: t!("admin-scenarios-name-placeholder"),
                        r#type: "text",
                        value: "{edit_name}",
                        oninput: move |e: FormEvent| edit_name.set(e.value()),
                    }
                    Label {
                        html_for: "scenario-desc",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        {t!("admin-scenarios-col-description")}
                    }
                    Input {
                        placeholder: t!("admin-scenarios-description-placeholder"),
                        r#type: "text",
                        value: "{edit_description}",
                        oninput: move |e: FormEvent| edit_description.set(e.value()),
                    }
                    Label {
                        html_for: "scenario-level",
                        color: "var(--rpg-text-muted)",
                        font_size: "0.82rem",
                        {t!("admin-scenarios-level-label")}
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
                        {t!("admin-scenarios-bosses-label")}
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
                        {t!("admin-scenarios-loots-label")}
                    }
                    div { style: "display:flex;flex-direction:column;gap:8px;",
                        for (idx, loot) in edit_loots().iter().enumerate() {
                            {
                                let loot = loot.clone();
                                let idx_rm = idx;
                                rsx! {
                                    div { class: "loot-row",
                                        Input {
                                            placeholder: t!("admin-equip-name-label"),
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
                                            option { value: "Equipment", selected: loot.kind == "Equipment", {t!("loot-kind-equipment")} }
                                            option { value: "Consumable", selected: loot.kind == "Consumable",
                                                {t!("loot-kind-consumable")}
                                            }
                                            option { value: "Material", selected: loot.kind == "Material", {t!("loot-kind-material")} }
                                            option { value: "Currency", selected: loot.kind == "Currency", {t!("loot-kind-currency")} }
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
                                            option { value: "Common", selected: loot.rank == "Common", {t!("rank-common")} }
                                            option { value: "Intermediate", selected: loot.rank == "Intermediate",
                                                {t!("rank-intermediate")}
                                            }
                                            option { value: "Advanced", selected: loot.rank == "Advanced", {t!("rank-advanced")} }
                                        }
                                        Input {
                                            placeholder: t!("admin-scenarios-loot-level-placeholder"),
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
                                            placeholder: t!("admin-scenarios-loot-classes-placeholder"),
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
                                            {t!("admin-scenarios-remove-loot")}
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
                        {t!("admin-scenarios-add-loot")}
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
                                        feedback.set(t!("admin-scenarios-file-stem-empty"));
                                        return;
                                    }
                                    if detail.name.trim().is_empty() {
                                        feedback.set(t!("admin-equip-name-empty"));
                                        return;
                                    }
                                    match save_scenario_detail(u.clone(), fs, detail).await {
                                        Ok(()) => {
                                            feedback.set(t!("admin-equip-saved"));
                                            edit_mode.set(ScenarioEditMode::None);
                                            universes_resource.restart();
                                            if let Ok(mut s) = admin_list_scenarios().await {
                                                s.retain(|sc| sc.universe == u);
                                                s.sort_by_key(|sc| sc.level);
                                                scenarios.set(s);
                                            }
                                        }
                                        Err(e) => feedback.set(t!("admin-error", error : e.to_string())),
                                    }
                                });
                            },
                            {t!("admin-equip-save")}
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| {
                                edit_mode.set(ScenarioEditMode::None);
                                feedback.set(String::new());
                            },
                            {t!("common-cancel")}
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

use dioxus::prelude::*;

use crate::{
    auth_manager::server_fn::{
        EquipStatEntry, EquipmentFormData, admin_create_equipment, admin_delete_equipment,
        admin_get_equipment_form, admin_get_equipment_json, admin_list_equipment_categories,
        admin_list_equipment_items, admin_list_equipment_types, admin_save_equipment_form,
        admin_save_equipment_json,
    },
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        label::Label,
    },
};

#[component]
pub fn AdminEquipmentTab() -> Element {
    let mut eq_types: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_type = use_signal(String::new);
    let mut eq_categories: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_category = use_signal(String::new);
    let mut eq_items: Signal<Vec<String>> = use_signal(Vec::new);
    let mut selected_item: Signal<Option<String>> = use_signal(|| None);
    let mut eq_json = use_signal(String::new);
    let mut eq_form_mode = use_signal(|| true);
    let mut feedback = use_signal(String::new);
    let mut confirm_delete_item: Signal<Option<String>> = use_signal(|| None);
    let mut show_create_form = use_signal(|| false);
    let mut new_item_name = use_signal(String::new);

    // Equipment form signals
    let mut eq_nom = use_signal(String::new);
    let mut eq_nom_unique = use_signal(String::new);
    let mut eq_categorie = use_signal(String::new);
    let mut eq_stats: Signal<Vec<EquipStatEntry>> = use_signal(Vec::new);

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
                                                match admin_get_equipment_json(t.clone(), c.clone(), n.clone()).await {
                                                    Ok(json) => {
                                                        eq_json.set(json);
                                                        selected_item.set(Some(n.clone()));
                                                    }
                                                    Err(e) => {
                                                        feedback.set(format!("❌ {e}"));
                                                        return;
                                                    }
                                                }
                                                match admin_get_equipment_form(t, c, n).await {
                                                    Ok(form) => {
                                                        eq_nom.set(form.nom);
                                                        eq_nom_unique.set(form.nom_unique);
                                                        eq_categorie.set(form.categorie);
                                                        eq_stats.set(form.stats);
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

        // ── Create new equipment item ─────────────────────────────────────────
        if !selected_type().is_empty() && !selected_category().is_empty() {
            div { class: "admin-card",
                div { class: "eq-create-header",
                    p { class: "admin-section-title", style: "margin:0;",
                        "➕ New Item in {selected_category}"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            show_create_form.set(!show_create_form());
                            new_item_name.set(String::new());
                        },
                        if show_create_form() {
                            "✕ Cancel"
                        } else {
                            "➕ New"
                        }
                    }
                }
                if show_create_form() {
                    div { class: "eq-create-form",
                        div { class: "admin-form-field",
                            Label {
                                html_for: "eq-new-name",
                                color: "var(--rpg-text-muted)",
                                font_size: "0.82rem",
                                "Item filename (no spaces, no extension)"
                            }
                            Input {
                                id: "eq-new-name",
                                r#type: "text",
                                placeholder: "e.g. epic_sword",
                                value: "{new_item_name}",
                                oninput: move |e: FormEvent| new_item_name.set(e.value()),
                            }
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                let t = selected_type();
                                let c = selected_category();
                                let n = new_item_name().trim().to_owned();
                                if n.is_empty() {
                                    feedback.set("❌ Name cannot be empty.".to_owned());
                                    return;
                                }
                                spawn(async move {
                                    match admin_create_equipment(t.clone(), c.clone(), n.clone()).await {
                                        Ok(()) => {
                                            feedback.set(format!("✅ '{n}' created."));
                                            show_create_form.set(false);
                                            new_item_name.set(String::new());
                                            if let Ok(items) = admin_list_equipment_items(t.clone(), c.clone())
                                                .await
                                            {
                                                eq_items.set(items);
                                            }
                                            if let Ok(json) = admin_get_equipment_json(
                                                    t.clone(),
                                                    c.clone(),
                                                    n.clone(),
                                                )
                                                .await
                                            {
                                                eq_json.set(json);
                                                selected_item.set(Some(n.clone()));
                                            }
                                            if let Ok(form) = admin_get_equipment_form(t, c, n).await {
                                                eq_nom.set(form.nom);
                                                eq_nom_unique.set(form.nom_unique);
                                                eq_categorie.set(form.categorie);
                                                eq_stats.set(form.stats);
                                                eq_form_mode.set(true);
                                            }
                                        }
                                        Err(e) => feedback.set(format!("❌ {e}")),
                                    }
                                });
                            },
                            "💾 Create"
                        }
                    }
                }
            }
        }

        // ── Edit selected item ────────────────────────────────────────────────
        if let Some(item_name) = selected_item() {
            {
                let item_name_save = item_name.clone();
                let item_name_cancel = item_name.clone();
                rsx! {
                    div { class: "admin-full-card",
                        div { style: "display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;",
                            p { class: "admin-section-title", style: "margin:0;", "✏️ {item_name}" }
                            Button {
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| eq_form_mode.set(!eq_form_mode()),
                                if eq_form_mode() {
                                    "✏️ JSON mode"
                                } else {
                                    "📝 Form mode"
                                }
                            }
                        }
                        if eq_form_mode() {
                            div { class: "admin-form-grid",
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "eq-nom",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Nom"
                                    }
                                    Input {
                                        r#type: "text",
                                        value: "{eq_nom}",
                                        oninput: move |e: FormEvent| eq_nom.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "eq-nom-unique",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Nom unique"
                                    }
                                    Input {
                                        r#type: "text",
                                        value: "{eq_nom_unique}",
                                        oninput: move |e: FormEvent| eq_nom_unique.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "eq-categorie",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Catégorie"
                                    }
                                    Input {
                                        r#type: "text",
                                        value: "{eq_categorie}",
                                        oninput: move |e: FormEvent| eq_categorie.set(e.value()),
                                    }
                                }
                            }
                            if !eq_stats().is_empty() {
                                p { style: "font-weight:600;margin:12px 0 6px;color:var(--rpg-text-muted);font-size:0.82rem;",
                                    "Stats"
                                }
                                div { class: "admin-stats-table",
                                    div { class: "admin-stats-header",
                                        span { class: "ast-col-name", "Stat" }
                                        span { class: "ast-col-val", "Value" }
                                        span { class: "ast-col-sep", "" }
                                        span { class: "ast-col-val", "%" }
                                    }
                                    for (idx, stat) in eq_stats().into_iter().enumerate() {
                                        div { class: "admin-stats-row",
                                            span { class: "ast-col-name", "{stat.stat_name}" }
                                            input {
                                                class: "ast-input",
                                                r#type: "number",
                                                value: "{stat.equip_value}",
                                                oninput: move |e: FormEvent| {
                                                    let mut stats = eq_stats();
                                                    if let Some(s) = stats.get_mut(idx) {
                                                        s.equip_value = e.value().trim().parse::<i64>().unwrap_or(0);
                                                    }
                                                    eq_stats.set(stats);
                                                },
                                            }
                                            span { class: "ast-col-sep", "/" }
                                            input {
                                                class: "ast-input",
                                                r#type: "number",
                                                value: "{stat.equip_percent}",
                                                oninput: move |e: FormEvent| {
                                                    let mut stats = eq_stats();
                                                    if let Some(s) = stats.get_mut(idx) {
                                                        s.equip_percent = e.value().trim().parse::<i64>().unwrap_or(0);
                                                    }
                                                    eq_stats.set(stats);
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                            div { style: "display:flex;gap:8px;margin-top:8px;",
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| {
                                        let t = selected_type();
                                        let c = selected_category();
                                        let n = item_name_save.clone();
                                        let form = EquipmentFormData {
                                            nom: eq_nom(),
                                            nom_unique: eq_nom_unique(),
                                            categorie: eq_categorie(),
                                            stats: eq_stats(),
                                        };
                                        spawn(async move {
                                            match admin_save_equipment_form(t, c, n, form).await {
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
                                        feedback.set(String::new());
                                    },
                                    "Cancel"
                                }
                            }
                        } else {
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
                                        let n = item_name_cancel.clone();
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

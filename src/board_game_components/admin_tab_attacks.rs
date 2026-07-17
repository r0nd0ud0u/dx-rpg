use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::{
    auth_manager::server_fn::{
        AttackFormData, admin_delete_attack, admin_get_attack_form, admin_get_attack_json,
        admin_list_attacks, admin_save_attack_form, admin_save_attack_json, upload_photo,
    },
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        label::Label,
    },
};

// JavaScript code to read an image file from the input with id 'atk-photo-file'.
// - Gets the selected file from the input element.
// - If no file is selected, sends null to Dioxus and exits.
// - Otherwise, reads the file as a DataURL (base64).
// - When reading is complete, extracts the base64 data and sends an object
//   containing the file name and the base64 data to Dioxus.
const JS_READ_ATK_PHOTO: &str = "const input = document.getElementById('atk-photo-file'); \
     const file = input && input.files && input.files[0]; \
     if (!file) { dioxus.send(null); return; } \
     const reader = new FileReader(); \
     reader.onload = function(ev) { \
         const b64 = ev.target.result.split(',')[1]; \
         dioxus.send({name: file.name, data: b64}); \
     }; \
     reader.readAsDataURL(file);";

/// Attack management panel for a single character.
/// `char_name`     — the character whose attacks are being edited.
/// `attacks_list`  — shared signal (owned by the parent character tab) so
///                   that newly-created / deleted attacks are visible there too.
#[component]
pub fn AdminAttacksPanel(char_name: String, attacks_list: Signal<Vec<String>>) -> Element {
    let mut edit_attack_name: Signal<Option<String>> = use_signal(|| None);
    let mut attack_edit_form_mode = use_signal(|| false);
    let mut attack_json = use_signal(String::new);
    let mut attack_feedback = use_signal(String::new);
    let mut new_attack_name = use_signal(String::new);

    // Attack form fields
    let mut atk_nom = use_signal(String::new);
    let mut atk_niveau = use_signal(|| "1".to_owned());
    let mut atk_description = use_signal(String::new);
    let mut atk_cible = use_signal(|| "Enemy".to_owned());
    let mut atk_portee = use_signal(|| "Individual".to_owned());
    let mut atk_forme = use_signal(|| "Standard".to_owned());
    let mut atk_cout_mana = use_signal(|| "0".to_owned());
    let mut atk_cout_rage = use_signal(|| "0".to_owned());
    let mut atk_cout_vigueur = use_signal(|| "0".to_owned());
    let mut atk_duree = use_signal(|| "1".to_owned());
    let mut atk_aggro = use_signal(|| "0".to_owned());
    let mut atk_degats = use_signal(|| "0".to_owned());
    let mut atk_soin = use_signal(|| "0".to_owned());
    let mut atk_regen_mana = use_signal(|| "0".to_owned());
    let mut atk_regen_rage = use_signal(|| "0".to_owned());
    let mut atk_regen_vigueur = use_signal(|| "0".to_owned());
    let mut atk_photo = use_signal(String::new);
    let mut atk_effet = use_signal(|| "[]".to_owned());

    // Photo upload handler
    let on_atk_photo_change = move |_: FormEvent| {
        let mut js = document::eval(JS_READ_ATK_PHOTO);
        spawn(async move {
            if let Ok(val) = js.recv::<serde_json::Value>().await
                && !val.is_null()
                && let (Some(name), Some(data)) = (
                    val.get("name")
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(String::from),
                    val.get("data")
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(String::from),
                )
            {
                let fname = name.clone();
                match upload_photo(name, data).await {
                    Ok(_) => atk_photo.set(fname),
                    Err(e) => {
                        attack_feedback.set(t!("admin-atk-upload-error", error: e.to_string()))
                    }
                }
            }
        });
    };

    rsx! {
        div { class: "admin-full-card",
            p { class: "admin-section-title",
                {t!("admin-atk-title", character : char_name.clone())}
            }

            if attacks_list().is_empty() {
                p { style: "color:var(--rpg-text-muted);", {t!("admin-atk-empty")} }
            } else {
                div { style: "display:flex;flex-wrap:wrap;gap:6px;margin-bottom:12px;",
                    for atk in attacks_list() {
                        {
                            let atk_name = atk.clone();
                            let c_load = char_name.clone();
                            rsx! {
                                Button {
                                    variant: if edit_attack_name() == Some(atk_name.clone()) { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                    onclick: move |_| {
                                        let n = atk_name.clone();
                                        let c = c_load.clone();
                                        attack_feedback.set(String::new());
                                        attack_edit_form_mode.set(false);
                                        spawn(async move {
                                            match admin_get_attack_json(c, n.clone()).await {
                                                Ok(json) => {
                                                    attack_json.set(json);
                                                    edit_attack_name.set(Some(n));
                                                }
                                                Err(e) => attack_feedback.set(t!("admin-error", error : e.to_string())),
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

            // ── Selected attack editor ──────────────────────────────────────
            if let Some(aname) = edit_attack_name() {
                {
                    let c_form_switch = char_name.clone();
                    let c_form_save = char_name.clone();
                    let c_json_switch = char_name.clone();
                    let c_json_save = char_name.clone();
                    let c_del_a = char_name.clone();
                    let c_del_b = char_name.clone();
                    let aname_form_switch = aname.clone();
                    let aname_form_save = aname.clone();
                    let aname_json_save = aname.clone();
                    let aname_del_a = aname.clone();
                    let aname_del_b = aname.clone();
                    rsx! {
                        if attack_edit_form_mode() {
                            div { style: "display:flex;align-items:center;justify-content:space-between;margin:8px 0 4px;",
                                p { style: "font-weight:600;", {t!("admin-atk-form-edit-title", name : aname.clone())} }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        let c = c_form_switch.clone();
                                        let n = aname_form_switch.clone();
                                        spawn(async move {
                                            if let Ok(json) = admin_get_attack_json(c, n).await {
                                                attack_json.set(json);
                                            }
                                        });
                                        attack_edit_form_mode.set(false);
                                    },
                                    {t!("admin-equip-json-mode")}
                                }
                            }
                            div { class: "admin-form-grid",
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-nom",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-equip-name-label")}
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
                                        {t!("admin-atk-level-label")}
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
                                        {t!("admin-atk-target-label")}
                                    }
                                    select {
                                        class: "admin-select",
                                        value: "{atk_cible}",
                                        onchange: move |e| atk_cible.set(e.value()),
                                        option { value: "Enemy", {t!("admin-atk-target-enemy")} }
                                        option { value: "Ally", {t!("admin-atk-target-ally")} }
                                        option { value: "Self", {t!("admin-atk-target-self")} }
                                        option { value: "Zone", {t!("admin-atk-target-zone")} }
                                        option { value: "All", {t!("admin-atk-target-all")} }
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-portee",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-reach-label")}
                                    }
                                    select {
                                        class: "admin-select",
                                        value: "{atk_portee}",
                                        onchange: move |e| atk_portee.set(e.value()),
                                        option { value: "Individual", {t!("admin-atk-reach-individual")} }
                                        option { value: "Area", {t!("admin-atk-reach-area")} }
                                        option { value: "All", {t!("admin-atk-reach-all")} }
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-forme",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-form-label")}
                                    }
                                    select {
                                        class: "admin-select",
                                        value: "{atk_forme}",
                                        onchange: move |e| atk_forme.set(e.value()),
                                        option { value: "Standard", {t!("admin-atk-form-standard")} }
                                        option { value: "Magic", {t!("admin-atk-form-magic")} }
                                        option { value: "Healing", {t!("admin-atk-form-healing")} }
                                        option { value: "Support", {t!("admin-atk-form-support")} }
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-photo",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-photo-label")}
                                    }
                                    Input {
                                        r#type: "text",
                                        placeholder: t!("admin-atk-photo-placeholder"),
                                        value: "{atk_photo}",
                                        oninput: move |e: FormEvent| atk_photo.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-photo-file",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-upload-photo-label")}
                                    }
                                    input {
                                        r#type: "file",
                                        id: "atk-photo-file",
                                        accept: "image/png,image/jpeg,image/webp,image/gif",
                                        onchange: on_atk_photo_change,
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-mana",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-cost-mana-label")}
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
                                        {t!("admin-atk-cost-rage-label")}
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
                                        {t!("admin-atk-cost-vigor-label")}
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
                                        {t!("admin-atk-duration-label")}
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
                                        {t!("admin-atk-aggro-label")}
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{atk_aggro}",
                                        oninput: move |e: FormEvent| atk_aggro.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-degats",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-damage-label")}
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{atk_degats}",
                                        oninput: move |e: FormEvent| atk_degats.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-soin",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-heal-label")}
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{atk_soin}",
                                        oninput: move |e: FormEvent| atk_soin.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-regen-mana",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-regen-mana-label")}
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{atk_regen_mana}",
                                        oninput: move |e: FormEvent| atk_regen_mana.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-regen-rage",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-regen-rage-label")}
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{atk_regen_rage}",
                                        oninput: move |e: FormEvent| atk_regen_rage.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "atk-regen-vigueur",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        {t!("admin-atk-regen-vigor-label")}
                                    }
                                    Input {
                                        r#type: "number",
                                        value: "{atk_regen_vigueur}",
                                        oninput: move |e: FormEvent| atk_regen_vigueur.set(e.value()),
                                    }
                                }
                            }
                            Label {
                                html_for: "atk-description",
                                color: "var(--rpg-text-muted)",
                                font_size: "0.82rem",
                                {t!("admin-scenarios-col-description")}
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
                                {t!("admin-atk-effects-label")}
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
                                        let c = c_form_save.clone();
                                        let n = aname_form_save.clone();
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
                                            degats: atk_degats().trim().parse::<i64>().unwrap_or(0),
                                            soin: atk_soin().trim().parse::<i64>().unwrap_or(0),
                                            regen_mana: atk_regen_mana().trim().parse::<i64>().unwrap_or(0),
                                            regen_rage: atk_regen_rage().trim().parse::<i64>().unwrap_or(0),
                                            regen_vigueur: atk_regen_vigueur().trim().parse::<i64>().unwrap_or(0),
                                            photo: atk_photo(),
                                            effet_json: atk_effet(),
                                        };
                                        spawn(async move {
                                            match admin_save_attack_form(c, n, form).await {
                                                Ok(()) => attack_feedback.set(t!("admin-equip-saved")),
                                                Err(e) => attack_feedback.set(t!("admin-error", error : e.to_string())),
                                            }
                                        });
                                    },
                                    {t!("admin-equip-save")}
                                }
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| {
                                        let c = c_del_a.clone();
                                        let n = aname_del_a.clone();
                                        spawn(async move {
                                            match admin_delete_attack(c.clone(), n).await {
                                                Ok(()) => {
                                                    attack_feedback.set(t!("admin-deleted"));
                                                    edit_attack_name.set(None);
                                                    if let Ok(list) = admin_list_attacks(c).await {
                                                        attacks_list.set(list);
                                                    }
                                                }
                                                Err(e) => attack_feedback.set(t!("admin-error", error : e.to_string())),
                                            }
                                        });
                                    },
                                    {t!("admin-scenarios-delete")}
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        edit_attack_name.set(None);
                                        attack_feedback.set(String::new());
                                    },
                                    {t!("common-cancel")}
                                }
                            }
                        } else {
                            div { style: "display:flex;align-items:center;justify-content:space-between;margin:8px 0 4px;",
                                p { style: "font-weight:600;", "✏️ {aname}" }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        let c = c_json_switch.clone();
                                        let n = aname.clone();
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
                                                    atk_degats.set(form.degats.to_string());
                                                    atk_soin.set(form.soin.to_string());
                                                    atk_regen_mana.set(form.regen_mana.to_string());
                                                    atk_regen_rage.set(form.regen_rage.to_string());
                                                    atk_regen_vigueur.set(form.regen_vigueur.to_string());
                                                    atk_photo.set(form.photo);
                                                    atk_effet.set(form.effet_json);
                                                    attack_edit_form_mode.set(true);
                                                }
                                                Err(e) => attack_feedback.set(t!("admin-error", error : e.to_string())),
                                            }
                                        });
                                    },
                                    {t!("admin-equip-form-mode")}
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
                                        let c = c_json_save.clone();
                                        let n = aname_json_save.clone();
                                        let json = attack_json();
                                        spawn(async move {
                                            match admin_save_attack_json(c, n, json).await {
                                                Ok(()) => attack_feedback.set(t!("admin-equip-saved")),
                                                Err(e) => attack_feedback.set(t!("admin-error", error : e.to_string())),
                                            }
                                        });
                                    },
                                    {t!("admin-equip-save")}
                                }
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| {
                                        let c = c_del_b.clone();
                                        let n = aname_del_b.clone();
                                        spawn(async move {
                                            match admin_delete_attack(c.clone(), n).await {
                                                Ok(()) => {
                                                    attack_feedback.set(t!("admin-deleted"));
                                                    edit_attack_name.set(None);
                                                    if let Ok(list) = admin_list_attacks(c).await {
                                                        attacks_list.set(list);
                                                    }
                                                }
                                                Err(e) => attack_feedback.set(t!("admin-error", error : e.to_string())),
                                            }
                                        });
                                    },
                                    {t!("admin-scenarios-delete")}
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| {
                                        edit_attack_name.set(None);
                                        attack_feedback.set(String::new());
                                    },
                                    {t!("common-cancel")}
                                }
                            }
                        }
                    }
                }
            }

            // ── New Attack ──────────────────────────────────────────────────
            div { style: "margin-top:16px;border-top:1px solid var(--rpg-border);padding-top:12px;",
                p { style: "font-weight:600;margin-bottom:6px;", {t!("admin-atk-new-title")} }
                div { style: "display:flex;gap:8px;align-items:center;",
                    Input {
                        placeholder: t!("admin-atk-new-placeholder"),
                        r#type: "text",
                        value: "{new_attack_name}",
                        oninput: move |e: FormEvent| new_attack_name.set(e.value()),
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {
                            let n = new_attack_name().trim().to_owned();
                            let c = char_name.clone();
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
                                        attack_feedback.set(t!("admin-atk-created"));
                                        if let Ok(list) = admin_list_attacks(c).await {
                                            attacks_list.set(list);
                                        }
                                    }
                                    Err(e) => attack_feedback.set(t!("admin-error", error : e.to_string())),
                                }
                            });
                        },
                        {t!("admin-atk-create-button")}
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
}

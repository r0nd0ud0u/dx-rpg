use dioxus::logger::tracing;
use dioxus::prelude::*;

use crate::{
    auth_manager::server_fn::{
        AdminCharacterInfo, CharacterFormData, StatEntry, admin_create_universe,
        admin_get_character_form, admin_get_character_json, admin_list_attacks, admin_list_bosses,
        admin_list_characters, admin_save_character_form, admin_save_character_json,
        list_universes_server, upload_photo,
    },
    board_game_components::admin_tab_attacks::AdminAttacksPanel,
    common::photo_src,
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        label::Label,
    },
};

const JS_READ_CHAR_PHOTO: &str = "const input = document.getElementById('char-photo-file'); const file = input && input.files && input.files[0]; if (!file) { dioxus.send(null); return; } const reader = new FileReader(); reader.onload = function(ev) { const b64 = ev.target.result.split(',')[1]; dioxus.send({name: file.name, data: b64}); }; reader.readAsDataURL(file);";

#[component]
pub fn AdminCharactersTab() -> Element {
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

    // Attack management state (panel shown when Some)
    let mut attacks_char: Signal<Option<String>> = use_signal(|| None);
    let mut attacks_list: Signal<Vec<String>> = use_signal(Vec::new);

    // Universe creation
    let mut new_universe_name = use_signal(String::new);
    let mut universe_feedback = use_signal(String::new);

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

    let on_char_photo_change = move |_: FormEvent| {
        let mut js = document::eval(JS_READ_CHAR_PHOTO);
        spawn(async move {
            if let Ok(val) = js.recv::<serde_json::Value>().await
                && !val.is_null()
            {
                let name = val
                    .get("name")
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .map(String::from);
                let data = val
                    .get("data")
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .map(String::from);
                if let (Some(name), Some(data)) = (name, data) {
                    let full_name = name.clone();
                    match upload_photo(name, data).await {
                        Ok(_) => form_photo.set(full_name),
                        Err(e) => char_feedback.set(format!("❌ Upload: {e}")),
                    }
                }
            }
        });
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

        // Create new universe
        div { class: "admin-card",
            p { class: "admin-section-title", "🌍 Create Universe" }
            div { style: "display:flex;gap:8px;align-items:center;",
                Input {
                    placeholder: "Universe name (e.g. pokemon)",
                    r#type: "text",
                    value: "{new_universe_name}",
                    oninput: move |e: FormEvent| new_universe_name.set(e.value()),
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| {
                        let name = new_universe_name().trim().to_owned();
                        if name.is_empty() {
                            return;
                        }
                        spawn(async move {
                            match admin_create_universe(name).await {
                                Ok(()) => {
                                    universe_feedback.set("✅ Universe created.".to_owned());
                                    new_universe_name.set(String::new());
                                }
                                Err(e) => universe_feedback.set(format!("❌ {e}")),
                            }
                        });
                    },
                    "Create"
                }
            }
            if !universe_feedback().is_empty() {
                p { class: if universe_feedback().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                    "{universe_feedback}"
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
                                            src: photo_src(&c.photo_name),
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
                                                spawn(async move {
                                                    match admin_list_attacks(n.clone()).await {
                                                        Ok(list) => {
                                                            attacks_list.set(list);
                                                            attacks_char.set(Some(n));
                                                            edit_char_name.set(None);
                                                        }
                                                        Err(e) => tracing::error!("admin_list_attacks: {e}"),
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

        // ── Character editor ──────────────────────────────────────────────────
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
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-class",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Class"
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
                                        "Photo filename"
                                    }
                                    Input {
                                        placeholder: "e.g. Thalia.png",
                                        r#type: "text",
                                        value: "{form_photo}",
                                        oninput: move |e: FormEvent| form_photo.set(e.value()),
                                    }
                                }
                                div { class: "admin-form-field",
                                    Label {
                                        html_for: "char-photo-file",
                                        color: "var(--rpg-text-muted)",
                                        font_size: "0.82rem",
                                        "Upload Photo"
                                    }
                                    input {
                                        r#type: "file",
                                        id: "char-photo-file",
                                        accept: "image/png,image/jpeg,image/webp,image/gif",
                                        onchange: on_char_photo_change,
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
            }
        }

        // ── Attack panel (mutually exclusive with character editor) ───────────
        if let Some(achar) = attacks_char() {
            AdminAttacksPanel { char_name: achar, attacks_list }
        }

        if !char_feedback().is_empty() && edit_char_name().is_none()
            && attacks_char().is_none()
        {
            p { class: if char_feedback().starts_with('✅') { "admin-answer" } else { "admin-answer-error" },
                "{char_feedback}"
            }
        }
    }
}

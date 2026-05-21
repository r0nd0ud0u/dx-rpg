use dioxus::prelude::*;

/// Summary of one character (hero or boss) for the admin panel.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdminCharacterInfo {
    pub db_full_name: String,
    pub photo_name: String,
    pub class: String,
    pub level: u64,
    pub description: String,
    pub universe: String,
    pub stats: std::collections::HashMap<String, (u64, u64)>,
}

/// A single stat entry for the character form.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StatEntry {
    pub stat_name: String,
    pub current: i64,
    pub max: i64,
}

/// Key fields of a character for structured form editing.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CharacterFormData {
    pub name: String,
    pub class: String,
    pub level: u64,
    pub photo: String,
    pub char_type: String,
    pub stats: Vec<StatEntry>,
}

/// Returns the list of hero characters for the admin panel.
#[server]
pub async fn admin_list_characters() -> Result<Vec<AdminCharacterInfo>, ServerFnError> {
    use crate::common::DATA_MANAGER;
    use lib_rpg::character_mod::character::CharacterKind;
    let dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let infos = dm
        .all_heroes
        .iter()
        .filter(|c| c.kind == CharacterKind::Hero)
        .map(|c| AdminCharacterInfo {
            db_full_name: c.db_full_name.clone(),
            photo_name: c.photo_name.clone(),
            class: format!("{} {}", c.class.to_emoji(), c.class.to_str()),
            level: c.level,
            description: c.description.clone(),
            universe: c.universe.clone(),
            stats: c
                .stats
                .all_stats
                .iter()
                .map(|(k, v)| (k.clone(), (v.current, v.max)))
                .collect(),
        })
        .collect();
    Ok(infos)
}

/// Returns the list of boss characters for the admin panel.
#[server]
pub async fn admin_list_bosses() -> Result<Vec<AdminCharacterInfo>, ServerFnError> {
    use crate::common::DATA_MANAGER;
    let dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let infos = dm
        .all_bosses
        .iter()
        .map(|c| AdminCharacterInfo {
            db_full_name: c.db_full_name.clone(),
            photo_name: c.photo_name.clone(),
            class: format!("{} {}", c.class.to_emoji(), c.class.to_str()),
            level: c.level,
            description: c.description.clone(),
            universe: c.universe.clone(),
            stats: c
                .stats
                .all_stats
                .iter()
                .map(|(k, v)| (k.clone(), (v.current, v.max)))
                .collect(),
        })
        .collect();
    Ok(infos)
}

/// Returns list of available universe names (union of DATA_MANAGER + filesystem scan).
#[server]
pub async fn list_universes_server() -> Result<Vec<String>, ServerFnError> {
    use crate::common::{DATA_MANAGER, OFFLINE_PATH};
    use std::path::Path;
    let dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let mut universes: std::collections::HashSet<String> = dm
        .list_universes()
        .into_iter()
        .chain(dm.list_hero_universes())
        .collect();
    drop(dm);
    for subdir in &["scenarios", "characters"] {
        let dir = Path::new(OFFLINE_PATH).join(subdir);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(name) = p.file_name() {
                        let s = name.to_string_lossy().to_string();
                        if !s.is_empty() {
                            universes.insert(s);
                        }
                    }
                }
            }
        }
    }
    let mut result: Vec<String> = universes.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Returns the raw JSON of a character file for the admin editor.
#[server]
pub async fn admin_get_character_json(
    universe: String,
    character_name: String,
) -> Result<String, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("characters")
        .join(&universe)
        .join(format!("{character_name}.json"));
    std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))
}

/// Saves the raw JSON of a character file (validates JSON first) and reloads DATA_MANAGER.
#[server]
pub async fn admin_save_character_json(
    universe: String,
    character_name: String,
    json_content: String,
) -> Result<(), ServerFnError> {
    use crate::common::{DATA_MANAGER, OFFLINE_PATH};
    use std::path::Path;
    serde_json::from_str::<serde_json::Value>(&json_content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let dir = Path::new(OFFLINE_PATH).join("characters").join(&universe);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir {dir:?}: {e}")))?;
    let path = dir.join(format!("{character_name}.json"));
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))?;
    let mut dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let offline_root = dm.offline_root.clone();
    dm.all_heroes.clear();
    dm.all_bosses.clear();
    dm.load_all_characters(&offline_root)
        .map_err(|e| ServerFnError::new(format!("Reload failed: {e}")))?;
    Ok(())
}

/// Returns the key fields of a character for form-based editing.
#[server]
pub async fn admin_get_character_form(
    universe: String,
    character_name: String,
) -> Result<CharacterFormData, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("characters")
        .join(&universe)
        .join(format!("{character_name}.json"));
    let content = std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let name = v["Name"].as_str().unwrap_or("").to_owned();
    let class = v["Class"].as_str().unwrap_or("Standard").to_owned();
    let level = v["Level"].as_u64().unwrap_or(1);
    let photo = v["Photo"].as_str().unwrap_or("").to_owned();
    let char_type = v["Type"].as_str().unwrap_or("Hero").to_owned();
    let stats = if let Some(stats_obj) = v["Stats"].as_object() {
        let mut s: Vec<StatEntry> = stats_obj
            .iter()
            .map(|(name, val)| StatEntry {
                stat_name: name.clone(),
                current: val["Current"].as_i64().unwrap_or(0),
                max: val["Max"].as_i64().unwrap_or(0),
            })
            .collect();
        s.sort_by(|a, b| a.stat_name.cmp(&b.stat_name));
        s
    } else {
        Vec::new()
    };
    Ok(CharacterFormData { name, class, level, photo, char_type, stats })
}

/// Saves key character fields back into the JSON file, preserving other fields.
#[server]
pub async fn admin_save_character_form(
    universe: String,
    character_name: String,
    form: CharacterFormData,
) -> Result<(), ServerFnError> {
    use crate::common::{DATA_MANAGER, OFFLINE_PATH};
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("characters")
        .join(&universe)
        .join(format!("{character_name}.json"));
    let content = std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))?;
    let mut v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    v["Name"] = serde_json::Value::String(form.name);
    v["Class"] = serde_json::Value::String(form.class);
    v["Level"] = serde_json::json!(form.level);
    v["Photo"] = serde_json::Value::String(form.photo);
    v["Type"] = serde_json::Value::String(form.char_type);
    for stat in &form.stats {
        v["Stats"][&stat.stat_name]["Current"] = serde_json::json!(stat.current);
        v["Stats"][&stat.stat_name]["Max"] = serde_json::json!(stat.max);
    }
    let json_content = serde_json::to_string_pretty(&v)
        .map_err(|e| ServerFnError::new(format!("Cannot serialize: {e}")))?;
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))?;
    let mut dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let offline_root = dm.offline_root.clone();
    dm.all_heroes.clear();
    dm.all_bosses.clear();
    dm.load_all_characters(&offline_root)
        .map_err(|e| ServerFnError::new(format!("Reload failed: {e}")))?;
    Ok(())
}

use dioxus::prelude::*;

/// Returns a list of top-level equipment type directories (e.g. "body", "characters").
#[post("/api/admin_list_equipment_types")]
pub async fn admin_list_equipment_types() -> Result<Vec<String>, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let dir = Path::new(OFFLINE_PATH).join("equipment");
    let mut types: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir()
                && let Some(name) = p.file_name()
            {
                types.push(name.to_string_lossy().to_string());
            }
        }
    }
    types.sort();
    Ok(types)
}

/// Returns category subdirectories for a given equipment type.
#[post("/api/admin_list_equipment_categories")]
pub async fn admin_list_equipment_categories(
    eq_type: String,
) -> Result<Vec<String>, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..") || eq_type.contains('/') || eq_type.contains('\\') {
        return Err(ServerFnError::new("Invalid equipment type path".to_owned()));
    }
    let dir = Path::new(OFFLINE_PATH).join("equipment").join(&eq_type);
    let mut categories: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir()
                && let Some(name) = p.file_name()
            {
                categories.push(name.to_string_lossy().to_string());
            }
        }
    }
    categories.sort();
    Ok(categories)
}

/// Returns the list of equipment item stems for a given type and category.
#[post("/api/admin_list_equipment_items")]
pub async fn admin_list_equipment_items(
    eq_type: String,
    category: String,
) -> Result<Vec<String>, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..")
        || eq_type.contains('/')
        || eq_type.contains('\\')
        || category.contains("..")
        || category.contains('/')
        || category.contains('\\')
    {
        return Err(ServerFnError::new("Invalid path component".to_owned()));
    }
    let dir = Path::new(OFFLINE_PATH)
        .join("equipment")
        .join(&eq_type)
        .join(&category);
    let mut items: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file()
                && p.extension().map(|x| x == "json").unwrap_or(false)
                && let Some(stem) = p.file_stem()
            {
                items.push(stem.to_string_lossy().to_string());
            }
        }
    }
    items.sort();
    Ok(items)
}

/// Returns the raw JSON of an equipment item file.
#[post("/api/admin_get_equipment_json")]
pub async fn admin_get_equipment_json(
    eq_type: String,
    category: String,
    item_name: String,
) -> Result<String, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..") || category.contains("..") || item_name.contains("..") {
        return Err(ServerFnError::new("Invalid path".to_owned()));
    }
    let path = Path::new(OFFLINE_PATH)
        .join("equipment")
        .join(&eq_type)
        .join(&category)
        .join(format!("{item_name}.json"));
    std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))
}

/// Saves the raw JSON of an equipment item file (validates JSON first).
#[post("/api/admin_save_equipment_json")]
pub async fn admin_save_equipment_json(
    eq_type: String,
    category: String,
    item_name: String,
    json_content: String,
) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..") || category.contains("..") || item_name.contains("..") {
        return Err(ServerFnError::new("Invalid path".to_owned()));
    }
    serde_json::from_str::<serde_json::Value>(&json_content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let dir = Path::new(OFFLINE_PATH)
        .join("equipment")
        .join(&eq_type)
        .join(&category);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir: {e}")))?;
    let path = dir.join(format!("{item_name}.json"));
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))
}

/// Deletes an equipment item file.
#[post("/api/admin_delete_equipment")]
pub async fn admin_delete_equipment(
    eq_type: String,
    category: String,
    item_name: String,
) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..") || category.contains("..") || item_name.contains("..") {
        return Err(ServerFnError::new("Invalid path".to_owned()));
    }
    let path = Path::new(OFFLINE_PATH)
        .join("equipment")
        .join(&eq_type)
        .join(&category)
        .join(format!("{item_name}.json"));
    std::fs::remove_file(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot delete {path:?}: {e}")))
}

/// Creates a new equipment item, copying the stats template from an existing
/// item in the same category (so every stat key is present at zero).
/// Returns `Err` if the item already exists.
#[post("/api/admin_create_equipment")]
pub async fn admin_create_equipment(
    eq_type: String,
    category: String,
    item_name: String,
) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..")
        || eq_type.contains('/')
        || eq_type.contains('\\')
        || category.contains("..")
        || category.contains('/')
        || category.contains('\\')
        || item_name.is_empty()
        || item_name.contains("..")
        || item_name.contains('/')
        || item_name.contains('\\')
    {
        return Err(ServerFnError::new("Invalid path component".to_owned()));
    }
    let dir = Path::new(OFFLINE_PATH)
        .join("equipment")
        .join(&eq_type)
        .join(&category);
    let dest = dir.join(format!("{item_name}.json"));
    if dest.exists() {
        return Err(ServerFnError::new(format!(
            "Item '{item_name}' already exists"
        )));
    }
    // Build a stat template by reading an existing item, or use a default set.
    let template_stats: serde_json::Map<String, serde_json::Value> =
        if let Ok(entries) = std::fs::read_dir(&dir) {
            entries
                .flatten()
                .find(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
                .and_then(|e| std::fs::read_to_string(e.path()).ok())
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| {
                    v["Stats"].as_object().map(|obj| {
                        obj.iter()
                            .map(|(k, _)| {
                                (
                                    k.clone(),
                                    serde_json::json!({"equip_value": 0, "equip_percent": 0}),
                                )
                            })
                            .collect()
                    })
                })
                .unwrap_or_default()
        } else {
            serde_json::Map::new()
        };
    let new_item = serde_json::json!({
        "Categorie": category,
        "Nom": item_name,
        "Nom unique": item_name,
        "Stats": template_stats,
    });
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir: {e}")))?;
    let content = serde_json::to_string_pretty(&new_item)
        .map_err(|e| ServerFnError::new(format!("Serialize error: {e}")))?;
    std::fs::write(&dest, content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {dest:?}: {e}")))
}

/// Returns a list of available image filenames.
/// Reads from PHOTOS_PATH env var (default: "photos").
#[post("/api/list_available_images")]
pub async fn list_available_images() -> Result<Vec<String>, ServerFnError> {
    let photos_dir = std::env::var("PHOTOS_PATH").unwrap_or_else(|_| "photos".to_owned());
    let mut names: Vec<String> = match std::fs::read_dir(&photos_dir) {
        Ok(entries) => entries
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|x| x == "png" || x == "jpg" || x == "jpeg" || x == "webp" || x == "gif")
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                e.path()
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    Ok(names)
}

// ── Equipment form structs ────────────────────────────────────────────────────

/// A single equipment stat bonus entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EquipStatEntry {
    pub stat_name: String,
    pub equip_value: i64,
    pub equip_percent: i64,
}

/// Key fields of an equipment item for structured form editing.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EquipmentFormData {
    pub nom: String,
    pub nom_unique: String,
    pub categorie: String,
    pub stats: Vec<EquipStatEntry>,
}

/// Returns the key fields of an equipment item for form-based editing.
#[post("/api/admin_get_equipment_form")]
pub async fn admin_get_equipment_form(
    eq_type: String,
    category: String,
    item_name: String,
) -> Result<EquipmentFormData, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..") || category.contains("..") || item_name.contains("..") {
        return Err(ServerFnError::new("Invalid path".to_owned()));
    }
    let path = Path::new(OFFLINE_PATH)
        .join("equipment")
        .join(&eq_type)
        .join(&category)
        .join(format!("{item_name}.json"));
    let content = std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let nom = v["Nom"].as_str().unwrap_or("").to_owned();
    let nom_unique = v["Nom unique"].as_str().unwrap_or("").to_owned();
    let categorie = v["Categorie"].as_str().unwrap_or("").to_owned();
    let stats = if let Some(stats_obj) = v["Stats"].as_object() {
        let mut entries: Vec<EquipStatEntry> = stats_obj
            .iter()
            .map(|(name, val)| EquipStatEntry {
                stat_name: name.clone(),
                equip_value: val["equip_value"].as_i64().unwrap_or(0),
                equip_percent: val["equip_percent"].as_i64().unwrap_or(0),
            })
            .collect();
        entries.sort_by(|a, b| a.stat_name.cmp(&b.stat_name));
        entries
    } else {
        Vec::new()
    };
    Ok(EquipmentFormData {
        nom,
        nom_unique,
        categorie,
        stats,
    })
}

/// Saves an equipment item from form fields, preserving any extra JSON fields.
#[post("/api/admin_save_equipment_form")]
pub async fn admin_save_equipment_form(
    eq_type: String,
    category: String,
    item_name: String,
    form: EquipmentFormData,
) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if eq_type.contains("..") || category.contains("..") || item_name.contains("..") {
        return Err(ServerFnError::new("Invalid path".to_owned()));
    }
    let dir = Path::new(OFFLINE_PATH)
        .join("equipment")
        .join(&eq_type)
        .join(&category);
    let path = dir.join(format!("{item_name}.json"));
    // Read existing JSON to preserve any extra fields, or build from scratch
    let mut v: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?
    } else {
        serde_json::json!({})
    };
    v["Nom"] = serde_json::Value::String(form.nom);
    v["Nom unique"] = serde_json::Value::String(form.nom_unique);
    v["Categorie"] = serde_json::Value::String(form.categorie);
    for stat in &form.stats {
        v["Stats"][&stat.stat_name]["equip_value"] = serde_json::json!(stat.equip_value);
        v["Stats"][&stat.stat_name]["equip_percent"] = serde_json::json!(stat.equip_percent);
    }
    let json_content = serde_json::to_string_pretty(&v)
        .map_err(|e| ServerFnError::new(format!("Cannot serialize: {e}")))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir: {e}")))?;
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))
}

// ── Universe creation ─────────────────────────────────────────────────────────

/// Creates a new universe directory under characters/ and scenarios/.
#[post("/api/admin_create_universe")]
pub async fn admin_create_universe(universe_name: String) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    if universe_name.is_empty()
        || universe_name.contains("..")
        || universe_name.contains('/')
        || universe_name.contains('\\')
    {
        return Err(ServerFnError::new("Invalid universe name".to_owned()));
    }
    for subdir in &["characters", "scenarios"] {
        let dir = Path::new(OFFLINE_PATH).join(subdir).join(&universe_name);
        std::fs::create_dir_all(&dir)
            .map_err(|e| ServerFnError::new(format!("Cannot create {dir:?}: {e}")))?;
    }
    Ok(())
}

// ── Photo upload ──────────────────────────────────────────────────────────────

/// Uploads a photo to the images directory.
/// `file_data_base64` must be a standard base64-encoded string of the image bytes.
/// The filename must have a valid image extension and no path separators.
#[post("/api/upload_photo")]
pub async fn upload_photo(
    file_name: String,
    file_data_base64: String,
) -> Result<String, ServerFnError> {
    use std::path::Path;
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') {
        return Err(ServerFnError::new("Invalid filename".to_owned()));
    }
    let ext = Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["png", "jpg", "jpeg", "webp", "gif"].contains(&ext.as_str()) {
        return Err(ServerFnError::new("Unsupported image format".to_owned()));
    }
    let bytes = data_encoding::BASE64
        .decode(file_data_base64.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Invalid base64: {e}")))?;
    let photos_dir = std::env::var("PHOTOS_PATH").unwrap_or_else(|_| "photos".to_owned());
    std::fs::create_dir_all(&photos_dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create photos dir: {e}")))?;
    let dest = Path::new(&photos_dir).join(&file_name);
    std::fs::write(&dest, &bytes)
        .map_err(|e| ServerFnError::new(format!("Cannot write photo: {e}")))?;
    Ok(file_name)
}

use dioxus::prelude::*;

/// Returns a list of top-level equipment type directories (e.g. "body", "characters").
#[server]
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
#[server]
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
#[server]
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
#[server]
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
#[server]
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
#[server]
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

/// Returns a list of available image filenames.
/// Reads from PHOTOS_PATH env var (default: "assets/img").
#[server]
pub async fn list_available_images() -> Result<Vec<String>, ServerFnError> {
    let photos_dir = std::env::var("PHOTOS_PATH").unwrap_or_else(|_| "assets/img".to_owned());
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

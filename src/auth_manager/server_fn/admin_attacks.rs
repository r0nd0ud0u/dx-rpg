use dioxus::prelude::*;

/// Key fields of an attack for structured form editing.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AttackFormData {
    pub nom: String,
    pub niveau: i64,
    pub description: String,
    pub cible: String,
    pub portee: String,
    pub forme: String,
    pub cout_mana: i64,
    pub cout_rage: i64,
    pub cout_vigueur: i64,
    pub duree: i64,
    pub aggro: i64,
    pub photo: String,
    /// Raw JSON array string for the complex Effet field.
    pub effet_json: String,
}

/// Returns the list of attack file stems for a given character.
#[server]
pub async fn admin_list_attacks(character_name: String) -> Result<Vec<String>, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let dir = Path::new(OFFLINE_PATH).join("attack").join(&character_name);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot read {dir:?}: {e}")))?
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
        })
        .collect();
    names.sort();
    Ok(names)
}

/// Returns the raw JSON of an attack file.
#[server]
pub async fn admin_get_attack_json(
    character_name: String,
    attack_name: String,
) -> Result<String, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("attack")
        .join(&character_name)
        .join(format!("{attack_name}.json"));
    std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))
}

/// Saves the raw JSON of an attack file (validates JSON first).
#[server]
pub async fn admin_save_attack_json(
    character_name: String,
    attack_name: String,
    json_content: String,
) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    serde_json::from_str::<serde_json::Value>(&json_content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let dir = Path::new(OFFLINE_PATH).join("attack").join(&character_name);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir {dir:?}: {e}")))?;
    let path = dir.join(format!("{attack_name}.json"));
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))
}

/// Deletes an attack file for a character.
#[server]
pub async fn admin_delete_attack(
    character_name: String,
    attack_name: String,
) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("attack")
        .join(&character_name)
        .join(format!("{attack_name}.json"));
    std::fs::remove_file(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot delete {path:?}: {e}")))
}

/// Returns the key fields of an attack for form-based editing.
#[server]
pub async fn admin_get_attack_form(
    character_name: String,
    attack_name: String,
) -> Result<AttackFormData, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("attack")
        .join(&character_name)
        .join(format!("{attack_name}.json"));
    let content = std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let effet_json = serde_json::to_string_pretty(
        v.get("Effet")
            .unwrap_or(&serde_json::Value::Array(Vec::new())),
    )
    .unwrap_or_else(|_| "[]".to_owned());
    Ok(AttackFormData {
        nom: v["Nom"].as_str().unwrap_or("").to_owned(),
        niveau: v["Niveau"].as_i64().unwrap_or(1),
        description: v["Description"].as_str().unwrap_or("").to_owned(),
        cible: v["Cible"].as_str().unwrap_or("Enemy").to_owned(),
        portee: v["Portée"].as_str().unwrap_or("Individual").to_owned(),
        forme: v["Forme"].as_str().unwrap_or("Standard").to_owned(),
        cout_mana: v["Coût de mana"].as_i64().unwrap_or(0),
        cout_rage: v["Coût de rage"].as_i64().unwrap_or(0),
        cout_vigueur: v["Coût de vigueur"].as_i64().unwrap_or(0),
        duree: v["Durée"].as_i64().unwrap_or(1),
        aggro: v["Aggro"].as_i64().unwrap_or(0),
        photo: v["Photo"].as_str().unwrap_or("").to_owned(),
        effet_json,
    })
}

/// Saves an attack from form fields, reconstructing the full JSON.
#[server]
pub async fn admin_save_attack_form(
    character_name: String,
    attack_name: String,
    form: AttackFormData,
) -> Result<(), ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let effet: serde_json::Value = serde_json::from_str(&form.effet_json)
        .unwrap_or_else(|_| serde_json::Value::Array(Vec::new()));
    let attack = serde_json::json!({
        "Nom": form.nom,
        "Niveau": form.niveau,
        "Description": form.description,
        "Cible": form.cible,
        "Portée": form.portee,
        "Forme": form.forme,
        "Coût de mana": form.cout_mana,
        "Coût de rage": form.cout_rage,
        "Coût de vigueur": form.cout_vigueur,
        "Durée": form.duree,
        "Aggro": form.aggro,
        "Photo": form.photo,
        "Effet": effet,
    });
    let json_content = serde_json::to_string_pretty(&attack)
        .map_err(|e| ServerFnError::new(format!("Cannot serialize: {e}")))?;
    let dir = Path::new(OFFLINE_PATH).join("attack").join(&character_name);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir {dir:?}: {e}")))?;
    let path = dir.join(format!("{attack_name}.json"));
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))
}

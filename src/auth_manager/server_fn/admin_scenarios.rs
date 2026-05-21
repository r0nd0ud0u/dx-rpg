use dioxus::prelude::*;

/// Summary of one scenario shown in the admin scenario list.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdminScenarioInfo {
    pub name: String,
    pub description: String,
    pub level: u64,
    pub nb_bosses: usize,
    pub file_name: String,
    pub universe: String,
}

/// Parses a scenario JSON file into AdminScenarioInfo.
#[cfg(feature = "server")]
fn parse_scenario_info(
    path: &std::path::Path,
    universe: &str,
) -> Option<AdminScenarioInfo> {
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let name = v["name"].as_str().unwrap_or("").to_owned();
    if name.is_empty() {
        return None;
    }
    let description = v["description"].as_str().unwrap_or("").to_owned();
    let level = v["level"].as_u64().unwrap_or(0);
    let nb_bosses = v["boss_patterns"]
        .as_object()
        .map(|o| o.len())
        .unwrap_or(0);
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Some(AdminScenarioInfo {
        name,
        description,
        level,
        nb_bosses,
        file_name,
        universe: universe.to_owned(),
    })
}

/// Returns the list of all scenarios by scanning the scenarios directory on disk.
#[server]
pub async fn admin_list_scenarios() -> Result<Vec<AdminScenarioInfo>, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let scenarios_dir = Path::new(OFFLINE_PATH).join("scenarios");
    let mut infos: Vec<AdminScenarioInfo> = Vec::new();

    // Top-level JSON files (no universe)
    if let Ok(entries) = std::fs::read_dir(&scenarios_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(info) = parse_scenario_info(&p, "") {
                    infos.push(info);
                }
            }
        }
    }

    // Universe sub-directories
    if let Ok(entries) = std::fs::read_dir(&scenarios_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let universe = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if let Ok(sub) = std::fs::read_dir(&p) {
                    for sub_entry in sub.flatten() {
                        let sp = sub_entry.path();
                        if sp.is_file()
                            && sp.extension().map(|e| e == "json").unwrap_or(false)
                        {
                            if let Some(info) = parse_scenario_info(&sp, &universe) {
                                infos.push(info);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(infos)
}

/// Returns sorted list of distinct universe names (empty string = no universe).
#[server]
pub async fn get_available_universes() -> Result<Vec<String>, ServerFnError> {
    use crate::common::DATA_MANAGER;
    let dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    Ok(dm.list_universes())
}

/// Returns scenario filenames (stems) for a given universe.
#[server]
pub async fn list_scenarios_for_universe(universe: String) -> Result<Vec<String>, ServerFnError> {
    use crate::common::{DATA_MANAGER, OFFLINE_PATH};
    use std::path::Path;
    let _dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let dir = Path::new(OFFLINE_PATH).join("scenarios").join(&universe);
    let entries = std::fs::read_dir(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot read {dir:?}: {e}")))?;
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x == "json")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
        })
        .collect();
    names.sort();
    Ok(names)
}

/// A single loot item for scenario editing.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScenarioLootItem {
    pub name: String,
    /// "Equipment" | "Consumable" | "Material" | "Currency"
    pub kind: String,
    /// "Common" | "Intermediate" | "Advanced"
    pub rank: String,
    pub level: i64,
    /// Comma-separated class names ("Standard", "Warrior", "Mage", "Healer", "Berserker")
    pub classes: String,
}

/// Structured form-friendly representation of a scenario for the admin edit form.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScenarioDetail {
    pub name: String,
    pub description: String,
    pub level: u64,
    /// One boss per line: "BossName" or "BossName: 0, 1, 2"
    pub boss_patterns_text: String,
    /// Structured loot items
    pub loots: Vec<ScenarioLootItem>,
}

/// Returns a structured ScenarioDetail for the admin edit form.
#[server]
pub async fn get_scenario_detail(
    universe: String,
    file_stem: String,
) -> Result<ScenarioDetail, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("scenarios")
        .join(&universe)
        .join(format!("{file_stem}.json"));
    let content = std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let name = v["name"].as_str().unwrap_or("").to_owned();
    let description = v["description"].as_str().unwrap_or("").to_owned();
    let level = v["level"].as_u64().unwrap_or(1);
    let boss_patterns_text = if let Some(obj) = v["boss_patterns"].as_object() {
        obj.iter()
            .map(|(boss, patterns)| {
                let idxs: Vec<String> = patterns
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_u64())
                            .map(|n| n.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                if idxs.is_empty() || (idxs.len() == 1 && idxs[0] == "0") {
                    boss.clone()
                } else {
                    format!("{}: {}", boss, idxs.join(", "))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };
    let loots = if let Some(arr) = v["loots"].as_array() {
        arr.iter()
            .map(|loot| ScenarioLootItem {
                name: loot["name"].as_str().unwrap_or("").to_owned(),
                kind: loot["kind"].as_str().unwrap_or("Equipment").to_owned(),
                rank: loot["rank"].as_str().unwrap_or("Common").to_owned(),
                level: loot["level"].as_i64().unwrap_or(1),
                classes: loot["classes"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|c| c.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "Standard".to_owned()),
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(ScenarioDetail {
        name,
        description,
        level,
        boss_patterns_text,
        loots,
    })
}

/// Saves a scenario from form fields (converts to JSON and writes to disk).
#[server]
pub async fn save_scenario_detail(
    universe: String,
    file_stem: String,
    detail: ScenarioDetail,
) -> Result<(), ServerFnError> {
    use crate::common::{DATA_MANAGER, OFFLINE_PATH};
    use std::path::Path;

    let mut boss_map = serde_json::Map::new();
    for line in detail.boss_patterns_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(colon_pos) = line.find(':') {
            let boss_name = line[..colon_pos].trim().to_owned();
            let patterns_str = line[colon_pos + 1..].trim();
            let patterns: Vec<serde_json::Value> = patterns_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .map(serde_json::Value::from)
                .collect();
            boss_map.insert(boss_name, serde_json::Value::Array(patterns));
        } else {
            boss_map.insert(
                line.to_owned(),
                serde_json::Value::Array(vec![serde_json::Value::from(0u64)]),
            );
        }
    }

    let loots_json_arr: Vec<serde_json::Value> = detail.loots.iter().map(|item| {
        let classes: Vec<serde_json::Value> = item.classes
            .split(',')
            .map(|c| serde_json::Value::String(c.trim().to_owned()))
            .filter(|v| v.as_str().map(|s| !s.is_empty()).unwrap_or(false))
            .collect();
        serde_json::json!({
            "name": item.name,
            "kind": item.kind,
            "rank": item.rank,
            "level": item.level,
            "classes": classes,
        })
    }).collect();

    let scenario = serde_json::json!({
        "name": detail.name,
        "description": detail.description,
        "level": detail.level,
        "boss_patterns": serde_json::Value::Object(boss_map),
        "loots": serde_json::Value::Array(loots_json_arr),
        "universe": universe,
    });

    let json_content = serde_json::to_string_pretty(&scenario)
        .map_err(|e| ServerFnError::new(format!("Cannot serialize: {e}")))?;

    let dir = Path::new(OFFLINE_PATH).join("scenarios").join(&universe);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir: {e}")))?;
    let path = dir.join(format!("{file_stem}.json"));
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write: {e}")))?;

    let mut dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let offline_root = dm.offline_root.clone();
    dm.all_scenarios.clear();
    dm.load_all_scenarios(&offline_root)
        .map_err(|e| ServerFnError::new(format!("Reload failed: {e}")))?;
    Ok(())
}

/// Returns full JSON content of a scenario file.
#[server]
pub async fn get_scenario_json(
    universe: String,
    file_stem: String,
) -> Result<String, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("scenarios")
        .join(&universe)
        .join(format!("{file_stem}.json"));
    std::fs::read_to_string(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot read {path:?}: {e}")))
}

/// Saves (creates or overwrites) a scenario JSON file and reloads the data manager.
#[server]
pub async fn save_scenario_json(
    universe: String,
    file_stem: String,
    json_content: String,
) -> Result<(), ServerFnError> {
    use crate::common::{DATA_MANAGER, OFFLINE_PATH};
    use std::path::Path;
    serde_json::from_str::<serde_json::Value>(&json_content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let dir = Path::new(OFFLINE_PATH).join("scenarios").join(&universe);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir {dir:?}: {e}")))?;
    let path = dir.join(format!("{file_stem}.json"));
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))?;
    let mut dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let offline_root = dm.offline_root.clone();
    dm.all_scenarios.clear();
    dm.load_all_scenarios(&offline_root)
        .map_err(|e| ServerFnError::new(format!("Reload failed: {e}")))?;
    Ok(())
}

/// Deletes a scenario JSON file and reloads the data manager.
#[server]
pub async fn delete_scenario_json(
    universe: String,
    file_stem: String,
) -> Result<(), ServerFnError> {
    use crate::common::{DATA_MANAGER, OFFLINE_PATH};
    use std::path::Path;
    let path = Path::new(OFFLINE_PATH)
        .join("scenarios")
        .join(&universe)
        .join(format!("{file_stem}.json"));
    std::fs::remove_file(&path)
        .map_err(|e| ServerFnError::new(format!("Cannot delete {path:?}: {e}")))?;
    let mut dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let offline_root = dm.offline_root.clone();
    dm.all_scenarios.clear();
    dm.load_all_scenarios(&offline_root)
        .map_err(|e| ServerFnError::new(format!("Reload failed: {e}")))?;
    Ok(())
}

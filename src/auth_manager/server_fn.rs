#[cfg(feature = "server")]
use crate::auth_manager::{auth::Session, auth::User, db::get_db, model::SqlUser};
use dioxus::{logger::tracing, prelude::*};
use std::collections::HashSet;

/// Returns whether password authentication is required, driven by the `USE_PASSWORD` env var.
#[server]
pub async fn get_use_password() -> Result<bool, ServerFnError> {
    Ok(std::env::var("USE_PASSWORD")
        .unwrap_or_else(|_| "false".to_string())
        .trim()
        .to_lowercase()
        == "true")
}

#[post("/api/user/login", auth: Session)]
pub async fn login(
    username: String,
    password: String,
    use_password: bool,
) -> Result<(), ServerFnError> {
    if username.trim() == "" || (password.is_empty() && use_password) {
        Err(ServerFnError::new(
            "Username or Password can't be empty!".to_owned(),
        ))
    } else {
        let pool = get_db().await;
        let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
            .bind(&username)
            .fetch_all(pool)
            .await
            .unwrap();

        if rows.is_empty() {
            Err(ServerFnError::new(format!(
                "Username {} is not registered!",
                username
            )))
        } else {
            let is_valid = bcrypt::verify(password, &rows[0].password).is_ok();
            if rows[0].is_connected {
                return Err(ServerFnError::new(
                    "That user is already connected.".to_owned(),
                ));
            }
            if !use_password || is_valid {
                tracing::info!("{}", format!("{:?}", rows[0].id));
                // update is_connected status in db
                match update_connection_status(username, true).await {
                    Ok(()) => {
                        auth.login_user(rows[0].id);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::info!("{}", e);
                        Err(ServerFnError::new(
                            "Fail to update connection status on db, abort login".to_owned(),
                        ))
                    }
                }
            } else {
                Err(ServerFnError::new("Password is not correct!".to_owned()))
            }
        }
    }
}

#[server]
pub async fn register(
    username: String,
    password: String,
    use_password: bool,
) -> Result<(), ServerFnError> {
    if username.trim() == "" || (password.is_empty() && use_password) {
        Err(ServerFnError::new(
            "Username or Password can't be empty!".to_owned(),
        ))
    } else {
        let pool = get_db().await;
        let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
            .bind(&username)
            .fetch_all(pool)
            .await
            .unwrap();
        if !rows.is_empty() {
            Err(ServerFnError::new(format!(
                "Username {} is already taken!",
                username
            )))
        } else if use_password {
            let hash_password = bcrypt::hash(password, 10).unwrap();
            match sqlx::query("INSERT INTO users (username, password) VALUES (?1, ?2)")
                .bind(&username)
                .bind(&hash_password)
                .execute(pool)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(ServerFnError::new(format!("{}", e))),
            }
        } else {
            match sqlx::query(
                "INSERT INTO users (anonymous, username, is_connected) VALUES (?1, ?2, ?3)",
            )
            .bind(false)
            .bind(&username)
            .bind(false)
            .execute(pool)
            .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(ServerFnError::new(format!("{}", e))),
            }
        }
    }
}

#[post("/api/user/delete_user")]
pub async fn delete_user(
    username: String,
    password: String,
    use_password: bool,
) -> Result<(), ServerFnError> {
    if username.trim() == "Admin" {
        let msg = "Admin cannot be deleted".to_owned();
        Err(ServerFnError::new(msg))
    } else if username.trim() == "" || (password.is_empty() && use_password) {
        let msg = "Username or Password can't be empty!".to_owned();
        Err(ServerFnError::new(msg))
    } else {
        let pool = get_db().await;
        let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
            .bind(&username)
            .fetch_all(pool)
            .await
            .unwrap();

        if rows.is_empty() {
            let msg = format!("Username {} is not registered!", username);
            Err(ServerFnError::new(msg))
        } else {
            let is_valid = bcrypt::verify(&password, &rows[0].password).is_ok();

            if use_password {
                if is_valid {
                    match sqlx::query("DELETE FROM users WHERE username = ?1 AND password = ?2")
                        .bind(&username)
                        .bind(&password)
                        .execute(pool)
                        .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ServerFnError::new(format!("{}", e))),
                    }
                } else {
                    Err(ServerFnError::new("Password is not correct!".to_owned()))
                }
            } else {
                match sqlx::query("DELETE FROM users WHERE username = ?1")
                    .bind(&username)
                    .execute(pool)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ServerFnError::new(format!("{}", e))),
                }
            }
        }
    }
}

/// Get the current user's permissions, guarding the endpoint with the `Auth` validator.
/// If this returns false, we use the `or_unauthorized` extension to return a 401 error.
#[get("/api/user/permissions", auth: Session)]
pub async fn get_permissions() -> Result<HashSet<String>> {
    use axum_session_auth::{Auth, Rights};

    let user = auth.current_user.unwrap();

    Auth::<User, i64, sqlx::SqlitePool>::build([axum::http::Method::GET], false)
        .requires(Rights::any([
            Rights::permission("Category::View"),
            Rights::permission("Admin::View"),
        ]))
        .validate(&user, &axum::http::Method::GET, None)
        .await
        .or_unauthorized("You do not have permission to view categories")?;

    Ok(user.permissions)
}

/// Just like `login`, but this time we log out the user.
#[post("/api/user/logout", auth: Session)]
pub async fn logout() -> Result<(), ServerFnError> {
    let name = match get_user_name().await {
        Ok(name) => name,
        Err(e) => return Err(ServerFnError::new(format!("{}", e))),
    };
    match update_connection_status(name, false).await {
        Ok(()) => {
            auth.logout_user();
            Ok(())
        }
        Err(_) => Err(ServerFnError::new("abord logout")),
    }
}

/// We can access the current user via `auth.current_user`.
/// We can have both anonymous user (id 1) and a logged in user (id 2).
///
/// Logged-in users will have more permissions which we can modify.
#[post("/api/user/name", auth: Session)]
pub async fn get_user_name() -> Result<String> {
    Ok(auth.current_user.unwrap().username)
}

#[post("/api/get/user/id", auth: Session)]
pub async fn get_user_id() -> Result<i64> {
    Ok(auth.current_user.unwrap().id)
}

#[post("/api/set/user", auth: Session)]
pub async fn set_user_by_id(user_id: i64) -> Result<()> {
    auth.login_user(user_id);
    Ok(())
}

#[cfg(feature = "server")]
#[server]
pub async fn update_connection_status(
    username: String,
    is_connected: bool,
) -> Result<(), ServerFnError> {
    let pool = get_db().await;
    tracing::info!(
        "UPDATE users SET is_connected = {} WHERE username = {}",
        is_connected,
        username
    );
    match sqlx::query("UPDATE users SET is_connected = ?1 WHERE username = ?2")
        .bind(is_connected)
        .bind(&username)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::new(format!("{}", e))),
    }
}

#[cfg(feature = "server")]
#[server]
pub async fn update_all_connection_status(is_connected: bool) -> Result<(), ServerFnError> {
    let pool = get_db().await;
    tracing::info!("UPDATE users SET is_connected = {}", is_connected,);
    match sqlx::query("UPDATE users SET is_connected = ?")
        .bind(is_connected)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::new(format!("{}", e))),
    }
}

/// Returns true if the Admin CRUD panel is enabled (controlled by `ADMIN_ENABLED` env var).
#[server]
pub async fn is_admin_enabled() -> Result<bool, ServerFnError> {
    Ok(std::env::var("ADMIN_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .trim()
        .to_lowercase()
        != "false")
}

/// Summary of one registered user shown in the admin user list.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdminUserInfo {
    pub username: String,
    pub is_connected: bool,
    pub nb_saves: usize,
}

/// Returns the list of all users with lightweight metadata, for the admin panel.
#[server]
pub async fn admin_list_users() -> Result<Vec<AdminUserInfo>, ServerFnError> {
    use crate::common::SAVED_DATA;
    use lib_rpg::{common::constants::paths_const::GAMES_DIR, utils::list_dirs_in_dir};

    let pool = get_db().await;
    let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let users = rows
        .into_iter()
        .map(|row| {
            let save_dir = SAVED_DATA.join(&row.username).join(GAMES_DIR.to_path_buf());
            let nb_saves = list_dirs_in_dir(&save_dir).map(|v| v.len()).unwrap_or(0);
            AdminUserInfo {
                username: row.username,
                is_connected: row.is_connected,
                nb_saves,
            }
        })
        .collect();
    Ok(users)
}

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

/// Summary of one hero character for the admin panel.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdminCharacterInfo {
    pub db_full_name: String,
    pub photo_name: String,
    pub class: String,
    pub level: u64,
    pub description: String,
    pub universe: String,
    pub stats: std::collections::HashMap<String, (u64, u64)>, // name -> (current, max)
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

/// Get a user setting value (returns default_val if not set).
#[server]
pub async fn get_user_setting(key: String, default_val: String) -> Result<String, ServerFnError> {
    let username = get_user_name()
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let pool = get_db().await;
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM user_settings WHERE username = ?1 AND key = ?2")
            .bind(&username)
            .bind(&key)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("{e}")))?;
    Ok(row.map(|(v,)| v).unwrap_or(default_val))
}

/// Save a user setting key/value pair.
#[server]
pub async fn save_user_setting(key: String, value: String) -> Result<(), ServerFnError> {
    let username = get_user_name()
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let pool = get_db().await;
    sqlx::query(
        "INSERT INTO user_settings (username, key, value) VALUES (?1, ?2, ?3)
         ON CONFLICT(username, key) DO UPDATE SET value = EXCLUDED.value",
    )
    .bind(&username)
    .bind(&key)
    .bind(&value)
    .execute(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("{e}")))?;
    Ok(())
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
    // Also scan the filesystem so universes show up even if DATA_MANAGER is empty
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
    /// One boss per line: "BossName" (defaults to pattern [0]) or "BossName: 0, 1, 2"
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

    // Build boss_patterns map from text
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

    // Convert structured loots to JSON array
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
    let loots = serde_json::Value::Array(loots_json_arr);

    let scenario = serde_json::json!({
        "name": detail.name,
        "description": detail.description,
        "level": detail.level,
        "boss_patterns": serde_json::Value::Object(boss_map),
        "loots": loots,
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

    // Reload DataManager
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
    // Validate JSON
    serde_json::from_str::<serde_json::Value>(&json_content)
        .map_err(|e| ServerFnError::new(format!("Invalid JSON: {e}")))?;
    let dir = Path::new(OFFLINE_PATH).join("scenarios").join(&universe);
    std::fs::create_dir_all(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot create dir {dir:?}: {e}")))?;
    let path = dir.join(format!("{file_stem}.json"));
    std::fs::write(&path, json_content.as_bytes())
        .map_err(|e| ServerFnError::new(format!("Cannot write {path:?}: {e}")))?;
    // Reload DataManager
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
    // Reload DataManager using offline_root to avoid relative-path drift
    let mut dm = DATA_MANAGER
        .lock()
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let offline_root = dm.offline_root.clone();
    dm.all_scenarios.clear();
    dm.load_all_scenarios(&offline_root)
        .map_err(|e| ServerFnError::new(format!("Reload failed: {e}")))?;
    Ok(())
}

// ─── Character & Attack admin server functions ────────────────────────────────

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
    // Reload heroes/bosses
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

/// Returns the list of attack file stems for a given character.
#[server]
pub async fn admin_list_attacks(
    character_name: String,
) -> Result<Vec<String>, ServerFnError> {
    use crate::common::OFFLINE_PATH;
    use std::path::Path;
    let dir = Path::new(OFFLINE_PATH)
        .join("attack")
        .join(&character_name);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| ServerFnError::new(format!("Cannot read {dir:?}: {e}")))?
        .flatten()
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
            .filter_map(|e| e.path().file_name().map(|n| n.to_string_lossy().to_string()))
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    Ok(names)
}

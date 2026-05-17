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
            let save_dir = SAVED_DATA
                .join(&row.username)
                .join(GAMES_DIR.to_path_buf());
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

/// Returns the list of all scenarios loaded in the data manager.
#[server]
pub async fn admin_list_scenarios() -> Result<Vec<AdminScenarioInfo>, ServerFnError> {
    use crate::common::DATA_MANAGER;
    let dm = DATA_MANAGER.lock().map_err(|e| ServerFnError::new(format!("{e}")))?;
    let infos = dm
        .all_scenarios
        .iter()
        .map(|s| {
            let file_name = if s.universe.is_empty() {
                format!("stage_{}.json", s.level)
            } else {
                format!("{}/stage_{}.json", s.universe, s.level)
            };
            AdminScenarioInfo {
                name: s.name.clone(),
                description: s.description.clone(),
                level: s.level,
                nb_bosses: s.boss_patterns.len(),
                file_name,
                universe: s.universe.clone(),
            }
        })
        .collect();
    Ok(infos)
}

/// Returns sorted list of distinct universe names (empty string = no universe).
#[server]
pub async fn get_available_universes() -> Result<Vec<String>, ServerFnError> {
    use crate::common::DATA_MANAGER;
    let dm = DATA_MANAGER.lock().map_err(|e| ServerFnError::new(format!("{e}")))?;
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
    pub stats: std::collections::HashMap<String, (u64, u64)>, // name -> (current, max)
}

/// Returns the list of hero characters for the admin panel.
#[server]
pub async fn admin_list_characters() -> Result<Vec<AdminCharacterInfo>, ServerFnError> {
    use crate::common::DATA_MANAGER;
    use lib_rpg::character_mod::character::CharacterKind;
    let dm = DATA_MANAGER.lock().map_err(|e| ServerFnError::new(format!("{e}")))?;
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
    let username = get_user_name().await.map_err(|e| ServerFnError::new(format!("{e}")))? ;
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
    let username = get_user_name().await.map_err(|e| ServerFnError::new(format!("{e}")))? ;
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

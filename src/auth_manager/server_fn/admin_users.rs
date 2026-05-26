#[cfg(feature = "server")]
use crate::auth_manager::{db::get_db, model::SqlUser};
use dioxus::prelude::*;

/// Returns true if the Admin CRUD panel is enabled (controlled by `ADMIN_ENABLED` env var).
#[server]
pub async fn is_admin_enabled() -> Result<bool, ServerFnError> {
    Ok(std::env::var("ADMIN_ENABLED")
        .unwrap_or_else(|_| "true".to_owned())
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

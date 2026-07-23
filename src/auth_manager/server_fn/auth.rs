#[cfg(feature = "server")]
use crate::auth_manager::{auth::Session, auth::User, db::get_db, model::SqlUser};
#[cfg(feature = "server")]
use dioxus::logger::tracing;
use dioxus::prelude::*;
#[cfg(feature = "server")]
use once_cell::sync::Lazy;
use std::collections::HashSet;
#[cfg(feature = "server")]
use std::{collections::HashMap, sync::Mutex};

/// Per-username, server-issued secret handed out only as `login()`'s return value once a real
/// login (password-checked or not, per `USE_PASSWORD`) has succeeded. The websocket's
/// `AddPlayer`/`LoginAllSessions` events require the caller to present this exact value (as
/// their `device_token`) before they're allowed to claim that username's live player slot —
/// without it, a raw websocket connection could otherwise claim to be any username just by
/// naming it, with no authentication at all. Overwritten on every fresh login, which is what
/// invalidates a stale/tampered client-side copy from an earlier session.
#[cfg(feature = "server")]
pub static LOGIN_PROOFS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Returns whether password authentication is required, driven by the `USE_PASSWORD` env var.
#[post("/api/get_use_password")]
pub async fn get_use_password() -> Result<bool, ServerFnError> {
    Ok(std::env::var("USE_PASSWORD")
        .unwrap_or_else(|_| "false".to_owned())
        .trim()
        .to_lowercase()
        == "true")
}

#[post("/api/user/login", auth: Session)]
pub async fn login(
    username: String,
    password: String,
    use_password: bool,
) -> Result<String, ServerFnError> {
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
            // Check the DB flag AND the real-time live-connection state: the DB flag can lag
            // behind (grace-period timing) or outlive (a crash that skipped clean disconnect)
            // the actual set of live websocket connections, and trusting it alone would let a
            // second real device log in as a user who is still actually connected elsewhere.
            let db_says_connected = rows[0].is_connected;
            let live_says_connected =
                crate::websocket_handler::event::is_username_connected(&username);
            tracing::info!(
                "login({}): db_is_connected={} live_is_connected={}",
                username,
                db_says_connected,
                live_says_connected
            );
            if db_says_connected || live_says_connected {
                return Err(ServerFnError::new(
                    "That user is already connected.".to_owned(),
                ));
            }
            if !use_password || is_valid {
                tracing::info!("{}", format!("{:?}", rows[0].id));
                match update_connection_status(username.clone(), true).await {
                    Ok(()) => {
                        auth.login_user(rows[0].id);
                        let proof = format!("{:032x}", rand::random::<u128>());
                        LOGIN_PROOFS.lock().unwrap().insert(username, proof.clone());
                        Ok(proof)
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

#[post("/api/register")]
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

#[post("/api/user/logout", auth: Session)]
pub async fn logout() -> Result<(), ServerFnError> {
    let name = match get_user_name().await {
        Ok(name) => name,
        Err(e) => return Err(ServerFnError::new(format!("{}", e))),
    };
    match update_connection_status(name.clone(), false).await {
        Ok(()) => {
            auth.logout_user();
            LOGIN_PROOFS.lock().unwrap().remove(&name);
            Ok(())
        }
        Err(_) => Err(ServerFnError::new("abord logout")),
    }
}

#[post("/api/user/name", auth: Session)]
pub async fn get_user_name() -> Result<String> {
    Ok(auth.current_user.unwrap().username)
}

#[post("/api/get/user/id", auth: Session)]
pub async fn get_user_id() -> Result<i64> {
    Ok(auth.current_user.unwrap().id)
}

#[cfg(feature = "server")]
#[post("/api/update_connection_status")]
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
#[post("/api/update_all_connection_status")]
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

/// Get a user setting value (returns default_val if not set).
#[post("/api/get_user_setting")]
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
#[post("/api/save_user_setting")]
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

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

/// Per-username secret, returned only by a successful `login()`. The websocket's
/// `AddPlayer`/`LoginAllSessions` require it as `device_token` before claiming a username's
/// player slot — otherwise a raw websocket could claim any username unauthenticated.
/// Overwritten on every login, which invalidates stale client copies.
#[cfg(feature = "server")]
pub static LOGIN_PROOFS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Brute-force/spam guard for `/api/user/login` and `/api/register`: allows 3 requests back to
/// back per client (normal mistyped-password tolerance), then replenishes 1 every 12s (~5/min
/// steady state).
///
/// Keyed per client IP, read from the `X-Forwarded-For` (falling back to `X-Real-IP`) header set
#[cfg(feature = "server")]
static AUTH_RATE_LIMITER: Lazy<governor::DefaultKeyedRateLimiter<String>> = Lazy::new(|| {
    governor::RateLimiter::keyed(
        governor::Quota::with_period(std::time::Duration::from_secs(12))
            .expect("12s is a valid quota period")
            .allow_burst(std::num::NonZeroU32::new(3).expect("3 is nonzero")),
    )
});

/// Extracts the client's IP from reverse-proxy headers for keying `AUTH_RATE_LIMITER`. See that
/// static's doc comment for why this — not `ConnectInfo` — is the source of truth here.
#[cfg(feature = "server")]
fn client_ip_key(headers: &axum::http::HeaderMap) -> String {
    // X-Forwarded-For can be a comma-separated chain (client, proxy1, proxy2, ...) — the
    // original client is the first entry.
    if let Some(first_ip) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|ip| !ip.is_empty())
    {
        return first_ip.to_owned();
    }
    if let Some(ip) = headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|ip| !ip.is_empty())
    {
        return ip.to_owned();
    }
    "unknown".to_owned()
}

/// Axum middleware wiring `AUTH_RATE_LIMITER` to just the login/register paths — see that
/// static's doc comment for the per-IP key extraction and its local-dev fallback.
#[cfg(feature = "server")]
pub async fn auth_rate_limit(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let path = req.uri().path();
    if (path == "/api/user/login" || path == "/api/register")
        && AUTH_RATE_LIMITER
            .check_key(&client_ip_key(req.headers()))
            .is_err()
    {
        return (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts, please wait a moment.",
        )
            .into_response();
    }
    next.run(req).await
}

/// Checks a typed password against the bcrypt hash stored in the `users.password` column.
///
/// `stored` is `None`/empty for accounts predating `USE_PASSWORD`: nothing to check, so
/// anything is accepted until the user sets one via `change_password()`.
///
/// `unwrap_or(false)`, not `.is_ok()`: `bcrypt::verify`'s `Err` means the *stored hash* was
/// malformed, so `.is_ok()` accepts any password against a well-formed hash.
#[cfg(feature = "server")]
fn password_matches(stored: Option<&str>, typed: &str) -> bool {
    match stored {
        Some(hash) if !hash.is_empty() => bcrypt::verify(typed, hash).unwrap_or(false),
        _ => true,
    }
}

/// Server-side source of truth for whether passwords are enforced, read straight from the
/// `USE_PASSWORD` env var.
///
/// Never use the client's `use_password` argument for an auth decision — it is a UI hint, and
/// a hand-rolled request can set it `false` to skip the check (or, on `/api/register`, store
/// no password at all, leaving a row that then matches anything).
#[cfg(feature = "server")]
fn use_password_enabled() -> bool {
    std::env::var("USE_PASSWORD")
        .unwrap_or_else(|_| "false".to_owned())
        .trim()
        .to_lowercase()
        == "true"
}

/// Returns whether password authentication is required, driven by the `USE_PASSWORD` env var.
#[post("/api/get_use_password")]
pub async fn get_use_password() -> Result<bool, ServerFnError> {
    Ok(use_password_enabled())
}

#[post("/api/user/login", auth: Session)]
pub async fn login(
    username: String,
    password: String,
    use_password: bool,
) -> Result<String, ServerFnError> {
    // Client-supplied value is a UI hint only — see `use_password_enabled`.
    let _ = use_password;
    let use_password = use_password_enabled();
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
            let is_valid = password_matches(rows[0].password.as_deref(), &password);
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
    // Client-supplied value is a UI hint only — see `use_password_enabled`.
    let _ = use_password;
    let use_password = use_password_enabled();
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
            match sqlx::query(
                "INSERT INTO users (anonymous, username, password, is_connected) VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(false)
            .bind(&username)
            .bind(&hash_password)
            .bind(false)
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

/// Sets or changes a password. `old_password` must match when one is already set; a legacy
/// account with none accepts anything, which is how it migrates onto a real password.
#[post("/api/user/change_password")]
pub async fn change_password(
    username: String,
    old_password: String,
    new_password: String,
    use_password: bool,
) -> Result<(), ServerFnError> {
    // Client-supplied value is a UI hint only — see `use_password_enabled`. Trusting it here
    // let a caller pass `false` and skip the old-password check entirely.
    let _ = use_password;
    let use_password = use_password_enabled();

    // `username` arrives as a request parameter, so it can name *any* account. Bind it to the
    // caller's own session: without this, an unauthenticated POST naming someone else's
    // username could set a new password for them and take the account over outright.
    let session_user = get_user_name().await.map_err(|_| {
        ServerFnError::new("You must be signed in to change a password.".to_owned())
    })?;
    if session_user != username {
        return Err(ServerFnError::new(
            "You can only change your own password.".to_owned(),
        ));
    }

    if new_password.trim().is_empty() {
        return Err(ServerFnError::new(
            "New password can't be empty!".to_owned(),
        ));
    }
    let pool = get_db().await;
    let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
        .bind(&username)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("{}", e)))?;

    let Some(row) = rows.into_iter().next() else {
        return Err(ServerFnError::new(format!(
            "Username {} is not registered!",
            username
        )));
    };

    let old_password_ok = !use_password || password_matches(row.password.as_deref(), &old_password);
    if !old_password_ok {
        return Err(ServerFnError::new(
            "Current password is not correct!".to_owned(),
        ));
    }

    let hash_password =
        bcrypt::hash(&new_password, 10).map_err(|e| ServerFnError::new(format!("{}", e)))?;
    sqlx::query("UPDATE users SET password = ?1 WHERE username = ?2")
        .bind(&hash_password)
        .bind(&username)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("{}", e)))?;
    Ok(())
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
            let is_valid = password_matches(rows[0].password.as_deref(), &password);

            if use_password {
                if is_valid {
                    // Username only — `is_valid` already checked the password. Binding the
                    // plaintext into the WHERE compared it against the *hash* column, so the
                    // DELETE matched 0 rows while still reporting Ok.
                    match sqlx::query("DELETE FROM users WHERE username = ?1")
                        .bind(&username)
                        .execute(pool)
                        .await
                    {
                        Ok(_) => {
                            cleanup_after_user_deletion(&username);
                            Ok(())
                        }
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
                    Ok(_) => {
                        cleanup_after_user_deletion(&username);
                        Ok(())
                    }
                    Err(e) => Err(ServerFnError::new(format!("{}", e))),
                }
            }
        }
    }
}

/// Runs the side effects of deleting a user account that aren't the `users` row itself:
/// force-logs-out any live session for them (so a currently-connected admin target doesn't
/// keep sitting in-game on a deleted account) and removes their saved games from disk.
#[cfg(feature = "server")]
fn cleanup_after_user_deletion(username: &str) {
    crate::websocket_handler::event::force_logout_user(username);

    let save_dir = crate::common::SAVED_DATA.join(username);
    if save_dir.exists()
        && let Err(e) = std::fs::remove_dir_all(&save_dir)
    {
        tracing::warn!(
            "cleanup_after_user_deletion: failed to remove save dir for {}: {}",
            username,
            e
        );
    }
}

#[get("/api/user/permissions", auth: Session)]
pub async fn get_permissions() -> Result<HashSet<String>> {
    use axum_session_auth::{Auth, Rights};

    let user = auth
        .current_user
        .or_unauthorized("no signed-in user in this session")?;

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

/// Whether `user` holds the "Admin::View" permission seeded for the Admin account alone
/// (see `db.rs`) — split out from `require_admin` below purely so it's unit-testable
/// against a plain `User` value, without needing a real `Session`/DB-backed auth session.
#[cfg(feature = "server")]
fn has_admin_permission(user: Option<&User>) -> bool {
    user.is_some_and(|user| user.permissions.contains("Admin::View"))
}

/// Server-side gate for every admin-only endpoint (`admin_*.rs`). The client only *hides* the
/// Admin Panel link, which is cosmetic — a raw HTTP request could call these directly. Checks
/// the session user against the "Admin::View" permission seeded in `db.rs`.
#[cfg(feature = "server")]
pub fn require_admin(auth: &Session) -> Result<(), ServerFnError> {
    if has_admin_permission(auth.current_user.as_ref()) {
        Ok(())
    } else {
        Err(ServerFnError::new(
            "Unauthorized: admin access required.".to_owned(),
        ))
    }
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

/// The signed-in user's name, or 401 when the request carries no session.
///
/// `current_user` is `None` whenever a client's local-storage username outlives the
/// server session (app update, restart, expired cookie) — unwrapping panicked the handler
/// on exactly the `logout` call meant to escape that state.
#[post("/api/user/name", auth: Session)]
pub async fn get_user_name() -> Result<String> {
    Ok(auth
        .current_user
        .or_unauthorized("no signed-in user in this session")?
        .username)
}

/// The signed-in user's id, or a 401. Same reasoning as [`get_user_name`].
#[post("/api/get/user/id", auth: Session)]
pub async fn get_user_id() -> Result<i64> {
    Ok(auth
        .current_user
        .or_unauthorized("no signed-in user in this session")?
        .id)
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

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    /// Cheapest bcrypt cost the crate accepts — these tests hash on every run and don't need
    /// the production cost of 10 to prove the comparison logic.
    const TEST_COST: u32 = 4;

    #[test]
    fn correct_password_is_accepted() {
        let hash = bcrypt::hash("hunter2", TEST_COST).unwrap();
        assert!(password_matches(Some(&hash), "hunter2"));
    }

    /// Regression test for the bug where all three call sites used
    /// `bcrypt::verify(..).is_ok()`. `verify` returns `Result<bool, _>` whose `Err` arm means
    /// "malformed hash", not "wrong password" — so `.is_ok()` was `true` for *any* password
    /// checked against a well-formed hash, letting anyone log in as anyone.
    #[test]
    fn wrong_password_is_rejected() {
        let hash = bcrypt::hash("hunter2", TEST_COST).unwrap();
        assert!(!password_matches(Some(&hash), "not-hunter2"));
        assert!(!password_matches(Some(&hash), ""));
        assert!(!password_matches(Some(&hash), "HUNTER2"));
    }

    /// A hash that `bcrypt::verify` can't even parse must fail closed, not open.
    #[test]
    fn malformed_hash_is_rejected() {
        assert!(!password_matches(Some("not-a-bcrypt-hash"), "anything"));
    }

    /// Documented legacy-account behaviour: a row written before `USE_PASSWORD` was enabled has
    /// no hash to compare against, so it stays reachable until its owner sets a real password.
    /// Admin/Guest are seeded this way in `db.rs`.
    #[test]
    fn account_without_stored_password_accepts_anything() {
        assert!(password_matches(None, "anything"));
        assert!(password_matches(Some(""), "anything"));
    }

    /// `USE_PASSWORD` must come from the server's own environment, never from the
    /// client-supplied argument that `login`/`register` accept purely as a UI hint.
    ///
    /// The only test touching the process environment. A second one must share a mutex with
    /// this: `set_var` is process-global and tests run in parallel threads.
    #[test]
    fn use_password_enabled_reads_the_env_var() {
        let previous = std::env::var("USE_PASSWORD").ok();
        // SAFETY: no other test reads or writes USE_PASSWORD (see doc comment above), so no
        // concurrent reader can observe these writes. The original value is restored below.
        unsafe {
            std::env::set_var("USE_PASSWORD", "true");
            assert!(use_password_enabled());
            std::env::set_var("USE_PASSWORD", " TRUE ");
            assert!(use_password_enabled(), "value is trimmed and lowercased");
            std::env::set_var("USE_PASSWORD", "false");
            assert!(!use_password_enabled());
            std::env::set_var("USE_PASSWORD", "yes");
            assert!(
                !use_password_enabled(),
                "only the literal \"true\" enables it"
            );
            std::env::remove_var("USE_PASSWORD");
            assert!(!use_password_enabled(), "defaults to false when unset");

            match previous {
                Some(v) => std::env::set_var("USE_PASSWORD", v),
                None => std::env::remove_var("USE_PASSWORD"),
            }
        }
    }

    fn user_with_permissions(perms: &[&str]) -> User {
        User {
            id: 1,
            anonymous: false,
            username: "Admin".to_owned(),
            permissions: perms.iter().map(|p| p.to_string()).collect(),
            is_connected: true,
        }
    }

    /// The actual gate every `admin_*` endpoint now runs behind — see `require_admin`'s
    /// doc comment for why this needed to exist at all (the endpoints had no server-side
    /// authorization check whatsoever before).
    #[test]
    fn admin_permission_required_and_sufficient() {
        assert!(has_admin_permission(Some(&user_with_permissions(&[
            "Admin::View"
        ]))));
    }

    #[test]
    fn non_admin_user_is_rejected() {
        assert!(!has_admin_permission(Some(&user_with_permissions(&[
            "Category::View"
        ]))));
        assert!(!has_admin_permission(Some(&user_with_permissions(&[]))));
    }

    #[test]
    fn no_current_user_is_rejected() {
        assert!(!has_admin_permission(None));
    }
}

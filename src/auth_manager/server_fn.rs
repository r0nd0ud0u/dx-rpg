#[cfg(feature = "server")]
use crate::auth_manager::{auth::Session, auth::User, db::get_db, model::SqlUser};
use dioxus::{logger::tracing, prelude::*};
use std::collections::HashSet;

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
            let result = match sqlx::query("INSERT INTO users (username, password) VALUES (?1, ?2)")
                .bind(&username)
                .bind(&hash_password)
                .execute(pool)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(ServerFnError::new(format!("{}", e))),
            };
            result
        } else {
            let result =
                match sqlx::query("INSERT INTO users (anonymous, username) VALUES (?1, ?2)")
                    .bind(false)
                    .bind(&username)
                    .execute(pool)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ServerFnError::new(format!("{}", e))),
                };
            result
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
                    let result = match sqlx::query(
                        "DELETE FROM users WHERE username = ?1 AND password = ?2",
                    )
                    .bind(&username)
                    .bind(&password)
                    .execute(pool)
                    .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ServerFnError::new(format!("{}", e))),
                    };
                    result
                } else {
                    Err(ServerFnError::new("Password is not correct!".to_owned()))
                }
            } else {
                let result = match sqlx::query("DELETE FROM users WHERE username = ?1")
                    .bind(&username)
                    .execute(pool)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ServerFnError::new(format!("{}", e))),
                };
                result
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

#[cfg(feature = "server")]
#[server]
pub async fn update_connection_status(
    username: String,
    is_connected: bool,
) -> Result<(), ServerFnError> {
    let pool = get_db().await;
    tracing::info!(
        "UPDATE users SET is_connected = {} WHERE username = {}",
        username,
        is_connected
    );
    match sqlx::query("UPDATE users SET is_connected = ?1 WHERE username = ?2")
        .bind(&is_connected)
        .bind(&username)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::new(format!("{}", e))),
    }
}

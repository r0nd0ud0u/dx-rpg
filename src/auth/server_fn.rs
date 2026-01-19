use std::collections::HashSet;
#[cfg(feature = "server")]
use crate::auth::{auth::Session, auth::User, model::SqlUser, db::get_db};
use dioxus::{logger::tracing, prelude::*};

#[post("/api/user/login", auth: Session)]
pub async fn login(
    username: String,
    password: String,
    use_password: bool,
) -> Result<(), ServerFnError> {
    if username.trim() == "" || (password.is_empty() && use_password) {
        let msg = format!("Username or Password can't be empty!");
        Err(ServerFnError::new(msg))
    } else {
        let pool = get_db().await;
        let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
            .bind(&username)
            .fetch_all(pool)
            .await
            .unwrap();

        if rows.len() == 0 {
            let msg = format!("Username {} is not registered!", username);
            Err(ServerFnError::new(msg))
        } else {
            let is_valid = match bcrypt::verify(password, &rows[0].password) {
                Ok(_) => true,
                _ => false,
            };

            if !use_password || (use_password && is_valid) {
                tracing::info!("{}", format!("{:?}", rows[0].id));
                auth.login_user(rows[0].id);
                Ok(())
            } else {
                let msg = format!("Password is not correct!");
                Err(ServerFnError::new(msg))
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
        let msg = format!("Username or Password can't be empty!");
        Err(ServerFnError::new(msg))
    } else {
        let pool = get_db().await;
        let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
            .bind(&username)
            .fetch_all(pool)
            .await
            .unwrap();
        if rows.len() != 0 {
            let msg = format!("Username  {} is already taken!", username);
            Err(ServerFnError::new(msg))
        } else {
            if use_password {
                let hash_password = bcrypt::hash(password, 10).unwrap();
                let result =
                    match sqlx::query("INSERT INTO users (username, password) VALUES (?1, ?2)")
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
}

#[post("/api/user/delete_user")]
pub async fn delete_user(
    username: String,
    password: String,
    use_password: bool,
) -> Result<(), ServerFnError> {
    if username.trim() == "Admin" {
        let msg = format!("Admin cannot be deleted");
        Err(ServerFnError::new(msg))
    }
    else if username.trim() == "" || (password.is_empty() && use_password) {
        let msg = format!("Username or Password can't be empty!");
        Err(ServerFnError::new(msg))
    } else {
        let pool = get_db().await;
        let rows: Vec<SqlUser> = sqlx::query_as("SELECT * FROM users WHERE username = ?1")
            .bind(&username)
            .fetch_all(pool)
            .await
            .unwrap();

        if rows.len() == 0 {
            let msg = format!("Username {} is not registered!", username);
            Err(ServerFnError::new(msg))
        } else {
            let is_valid = match bcrypt::verify(&password, &rows[0].password) {
                Ok(_) => true,
                _ => false,
            };

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
                    let msg = format!("Password is not correct!");
                    Err(ServerFnError::new(msg))
                }
            } else {
                let result =
                    match sqlx::query("DELETE FROM users WHERE username = ?1")
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
pub async fn logout() -> Result<()> {
    auth.logout_user();
    Ok(())
}

/// We can access the current user via `auth.current_user`.
/// We can have both anonymous user (id 1) and a logged in user (id 2).
///
/// Logged-in users will have more permissions which we can modify.
#[post("/api/user/name", auth: Session)]
pub async fn get_user_name() -> Result<String> {
    Ok(auth.current_user.unwrap().username)
}

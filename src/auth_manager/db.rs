#[cfg(feature = "server")]
use sqlx::{Executor, Pool, Sqlite};
#[cfg(feature = "server")]
use tokio::sync::OnceCell;

#[cfg(feature = "server")]
static DB: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

#[cfg(feature = "server")]
use std::env;

#[cfg(feature = "server")]
use dioxus::logger::tracing;

#[cfg(feature = "server")]
async fn db() -> Pool<Sqlite> {
    let db_url = match env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(e) => {
            tracing::error!("Error DATABASE_URL: {}", e);
            String::new()
        }
    };

    let pool = sqlx::sqlite::SqlitePool::connect(&db_url).await.unwrap();

    // Create the tables (sessions, users)
    pool.execute(r#"CREATE TABLE IF NOT EXISTS users ( "id" INTEGER PRIMARY KEY, "anonymous" BOOLEAN NOT NULL, "username" VARCHAR(256) NOT NULL, "password" VARCHAR(256), "is_connected" BOOLEAN NOT NULL)"#,)
            .await.unwrap();
    pool.execute(r#"CREATE TABLE IF NOT EXISTS user_permissions ( "user_id" INTEGER NOT NULL, "token" VARCHAR(256) NOT NULL)"#,)
            .await.unwrap();

    // Insert in some test data for two users (one anonymous, one normal)
    pool.execute(r#"INSERT INTO users (id, anonymous, username, password, is_connected) SELECT 1, true, 'Admin', '', false ON CONFLICT(id) DO UPDATE SET anonymous = EXCLUDED.anonymous, username = EXCLUDED.username, password = EXCLUDED.password, is_connected = EXCLUDED.is_connected"#,)
            .await.unwrap();
    pool.execute(r#"INSERT INTO users (id, anonymous, username, password, is_connected) SELECT 2, false, 'Guest', '', false ON CONFLICT(id) DO UPDATE SET anonymous = EXCLUDED.anonymous, username = EXCLUDED.username, password = EXCLUDED.password, is_connected = EXCLUDED.is_connected"#,)
            .await.unwrap();

    // permissions
    pool.execute(r#"INSERT INTO user_permissions (user_id, token) SELECT 1, 'Admin::View'"#)
        .await
        .unwrap();

    pool.execute(r#"INSERT INTO user_permissions (user_id, token) SELECT 2, 'Category::View'"#)
        .await
        .unwrap();

    pool
}

#[cfg(feature = "server")]
pub async fn get_db() -> &'static Pool<Sqlite> {
    DB.get_or_init(db).await
}

//! The code here is pulled from the `axum-session-auth` crate examples, requiring little to no
//! modification to work with dioxus fullstack.

use async_trait::async_trait;
use axum_session_auth::*;
use axum_session_sqlx::SessionSqlitePool;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::collections::HashSet;

use crate::auth_manager::model::SqlUser;

pub type Session = axum_session_auth::AuthSession<User, i64, SessionSqlitePool, SqlitePool>;
pub type AuthLayer = axum_session_auth::AuthSessionLayer<User, i64, SessionSqlitePool, SqlitePool>;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub anonymous: bool,
    pub username: String,
    pub permissions: HashSet<String>,
    pub is_connected: bool,
}
#[derive(sqlx::FromRow, Clone)]
pub struct SqlPermissionTokens {
    pub token: String,
}

#[async_trait]
impl Authentication<User, i64, SqlitePool> for User {
    async fn load_user(userid: i64, pool: Option<&SqlitePool>) -> Result<User, anyhow::Error> {
        let db = pool.unwrap();

        // `fetch_optional`, not `fetch_one`: `userid` here can be `ANONYMOUS_SESSION_USER_ID`
        // (see main.rs), which by construction never matches a row in `users` — every
        // cookie-less/never-logged-in request resolves to that id, so this must succeed
        // with an empty/anonymous `User` rather than erroring (this used to `.unwrap()`
        // straight into a panic on exactly that path before `ANONYMOUS_SESSION_USER_ID`
        // existed as a distinct id from any real account).
        let Some(sqluser) = sqlx::query_as::<_, SqlUser>("SELECT * FROM users WHERE id = $1")
            .bind(userid)
            .fetch_optional(db)
            .await?
        else {
            return Ok(User {
                id: userid,
                anonymous: true,
                username: String::new(),
                permissions: HashSet::new(),
                is_connected: false,
            });
        };

        //lets just get all the tokens the user can use, we will only use the full permissions if modifying them.
        let sql_user_perms = sqlx::query_as::<_, SqlPermissionTokens>(
            "SELECT token FROM user_permissions WHERE user_id = $1;",
        )
        .bind(userid)
        .fetch_all(db)
        .await
        .unwrap();

        Ok(User {
            id: sqluser.id,
            anonymous: sqluser.anonymous,
            username: sqluser.username,
            permissions: sql_user_perms.into_iter().map(|x| x.token).collect(),
            is_connected: sqluser.is_connected,
        })
    }

    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}

#[async_trait]
impl HasPermission<SqlitePool> for User {
    async fn has(&self, perm: &str, _pool: &Option<&SqlitePool>) -> bool {
        self.permissions.contains(perm)
    }
}

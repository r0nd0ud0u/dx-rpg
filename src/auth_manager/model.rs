#[derive(sqlx::FromRow, Clone)]
pub struct SqlUser {
    pub id: i64,
    pub anonymous: bool,
    pub username: String,
    pub password: String,
}

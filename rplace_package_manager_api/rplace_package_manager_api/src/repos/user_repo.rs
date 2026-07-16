use crate::{db::{db_provider::UserRepo, sqlite_db::SqliteDb}, models::user::user::{HashedUser, User}};
use anyhow::{Result};
use async_trait::async_trait;

#[async_trait]
impl UserRepo for SqliteDb {
    async fn new_user(&self, user: HashedUser) -> Result<User> {
        let sql = "INSERT INTO users (name,email,password_hash) VALUES (?,?,?) RETURNING *;";
        let user = sqlx::query_as::<_,User>(sql).bind(user.name).bind(user.email).bind(user.password_hash).fetch_one(&self.pool).await?;
        Ok(user)
    }

    async fn get_user_by_email(&self, email: String) -> Result<User> {
        let sql = "SELECT * FROM users WHERE email = ?;";
        let user = sqlx::query_as::<_,User>(sql).bind(email).fetch_one(&self.pool).await?;
        Ok(user)
    }

    async fn get_user_by_id(&self, id: i32) -> Result<User> {
        let sql = "SELECT * FROM users WHERE id = ?;";
        let user = sqlx::query_as::<_,User>(sql).bind(id).fetch_one(&self.pool).await?;
        Ok(user)
    }
}
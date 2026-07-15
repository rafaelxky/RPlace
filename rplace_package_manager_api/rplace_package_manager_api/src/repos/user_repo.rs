use crate::db::{db_provider::UserRepo, sqlite_db::SqliteDb};
use anyhow::{Result};
use async_trait::async_trait;

#[async_trait]
impl UserRepo for SqliteDb {
    async fn new_user(&self, user: crate::models::user::user::HashedUser) -> Result<crate::models::user::user::User> {
        todo!()
    }

    async fn get_user_by_email(&self, email: String) -> Result<crate::models::user::user::User> {
        todo!()
    }

    async fn get_user_by_id(&self, id: i32) -> Result<crate::models::user::user::User> {
        todo!()
    }
}
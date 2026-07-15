use crate::db::{db_provider::UserRepo, sqlite_db::SqliteDb};

#[async_trait]
impl UserRepo for SqliteDb {
    
}
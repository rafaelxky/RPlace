use async_trait::async_trait;

use crate::db::{db_provider::LinkRepo, sqlite_db::SqliteDb};

#[async_trait]
impl LinkRepo for SqliteDb {
    
}
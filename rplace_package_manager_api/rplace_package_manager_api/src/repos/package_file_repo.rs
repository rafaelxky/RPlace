use async_trait::async_trait;

use crate::db::{db_provider::PackageFileRepo, sqlite_db::SqliteDb};


#[async_trait]
impl PackageFileRepo for SqliteDb {
    
}
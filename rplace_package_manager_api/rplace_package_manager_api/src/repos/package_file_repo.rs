use async_trait::async_trait;
use anyhow::Result;

use crate::db::{db_provider::PackageFileRepo, sqlite_db::SqliteDb};


#[async_trait]
impl PackageFileRepo for SqliteDb {
    async fn get_package_file_by_hash(&self, file_hash: String) -> Result<crate::models::package_file::package_file::PackageFile> {
        todo!()
    }
}
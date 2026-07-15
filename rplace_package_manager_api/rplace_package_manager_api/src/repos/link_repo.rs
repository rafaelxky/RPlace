use async_trait::async_trait;
use anyhow::Result;

use crate::db::{db_provider::LinkRepo, sqlite_db::SqliteDb};

#[async_trait]
impl LinkRepo for SqliteDb {
    async fn get_link_by_package_version_id_and_file_path(&self, package_version_id: i32, file_path: String) -> Result<crate::models::link::link::Link> {
        todo!()
    }
}
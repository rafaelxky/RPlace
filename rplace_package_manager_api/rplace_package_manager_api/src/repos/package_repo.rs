use crate::{
    db::{db_provider::PackageVersionHeaderRepo, sqlite_db::SqliteDb},
};
use async_trait::async_trait;
use anyhow::{Result};

#[async_trait]
impl PackageVersionHeaderRepo for SqliteDb {
    async fn get_package_version_header_by_package_id_and_version(&self, package_id: i32, version: String) -> Result<crate::models::package_version_header::package_version_header::PackageVersionHeader> {
        todo!()
    }

    async fn get_latest_package_version_header_by_package_id(&self, package_id: i32) -> Result<crate::models::package_version_header::package_version_header::PackageVersionHeader> {
        todo!()
    }
}

use async_trait::async_trait;
use anyhow::{Result};

use crate::{db::{db_provider::PackageRegistryRepo, sqlite_db::SqliteDb}};

#[async_trait]
impl PackageRegistryRepo for SqliteDb {
    async fn get_registry_by_name(&self, name: String) -> Result<crate::models::registry::package_registry::PackageRegistry> {
        todo!()
    }

    async fn new_registry(&self, registry: crate::models::registry::package_registry::PackageRegistryCreateDto, user_id: i32) -> Result<crate::models::registry::package_registry::PackageRegistry> {
        todo!()
    }
}

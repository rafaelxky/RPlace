use async_trait::async_trait;
use anyhow::{Ok, Result};

use crate::{db::{db_provider::PackageRegistryRepo, sqlite_db::SqliteDb}, models::registry::package_registry::{PackageRegistry, PackageRegistryCreateDto}};

#[async_trait]
impl PackageRegistryRepo for SqliteDb {
    async fn get_registry_by_name(&self, name: String) -> Result<PackageRegistry>{
        let sql = "SELECT * FROM package_registry WHERE package_name = ?;";
        let registry = sqlx::query_as::<_,PackageRegistry>(sql).bind(name).fetch_one(&self.pool).await?;
        Ok(registry)
    }
    async fn get_registry_by_id(&self, id: i32) -> Result<PackageRegistry>{
        let sql = "SELECT * FROM package_registry WHERE id = ?;";
        let registry = sqlx::query_as::<_,PackageRegistry>(sql).bind(id).fetch_one(&self.pool).await?;
        Ok(registry)
    }
    async fn new_registry(&self, registry: PackageRegistryCreateDto, user_id: i32) -> Result<PackageRegistry>{
        let sql = "INSERT INTO package_registry (package_name, creator_id) VALUES (?,?) RETURNING *;";
        let registry = sqlx::query_as::<_, PackageRegistry>(sql).bind(registry.name).bind(user_id).fetch_one(&self.pool).await?;
        Ok(registry)
    }
}

use async_trait::async_trait;
use anyhow::{Ok, Result};

use crate::{db::{db_provider::PackageRegistryRepo, sqlite_db::SqliteDb}, models::package_registry::{PackageRegistry, PackageRegistryPublicDto}};

#[async_trait]
impl PackageRegistryRepo for SqliteDb {
    // creates new package registry
    async fn register_package(&self, name: String, creator_id: i32) -> Result<PackageRegistry> {
        let row: PackageRegistry =
            sqlx::query_as("INSERT INTO package_registry (name,creator_id) VALUES (?,?);")
                .bind(name)
                .bind(creator_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(row)
    }
    // gets package registry by NAME
    async fn get_package_registry_by_name(&self, name: String) -> Result<PackageRegistryPublicDto>{
      let sql = "SELECT * FROM package_registry WHERE name = ?;";
      let row: PackageRegistryPublicDto = sqlx::query_as(sql).bind(name).fetch_one(&self.pool).await?;
      Ok(row)
    }
    // delete all from package registry and packages by NAME
}

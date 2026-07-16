use crate::{
    db::{db_provider::PackageVersionHeaderRepo, sqlite_db::SqliteDb}, models::package_version_header::package_version_header::PackageVersionHeader,
};
use async_trait::async_trait;
use anyhow::{Ok, Result};

#[async_trait]
impl PackageVersionHeaderRepo for SqliteDb {
    async fn get_package_version_header_by_package_id_and_version(&self, package_id: i32, version: String) -> Result<PackageVersionHeader>{
        let sql = "SELECT * FROM package_version_header WHERE package_id = ? AND version = ?;";
        let header = sqlx::query_as::<_,PackageVersionHeader>(sql).bind(package_id).bind(version).fetch_one(&self.pool).await?;
        Ok(header)
    }
    async fn get_latest_package_version_header_by_package_id(&self, package_id: i32) -> Result<PackageVersionHeader>{
        let sql = "SELECT * FROM package_version_header WHERE package_id = ? ORDER BY created_at DESC LIMIT = 1;";
        let header = sqlx::query_as::<_,PackageVersionHeader>(sql).bind(package_id).fetch_one(&self.pool).await?;
        Ok(header)
    }
    async fn new_package_version(&self, version: String, package_id: i32) -> Result<PackageVersionHeader>{
        let sql = "INSERT INTO package_version_header (version, package_id) VALUES (?,?);";
        let header = sqlx::query_as::<_,PackageVersionHeader>(sql).bind(version).bind(package_id).fetch_one(&self.pool).await?;
        Ok(header)
    }
    async fn get_package_version_header_by_id(&self, id: i32) -> Result<PackageVersionHeader>{
        let sql = "SELECT * FROM package_version_header WHERE id = ?;";
        let header = sqlx::query_as::<_,PackageVersionHeader>(sql).bind(id).fetch_one(&self.pool).await?;
        Ok(header)
    }
}

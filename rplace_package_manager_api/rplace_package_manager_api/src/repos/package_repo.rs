use crate::{
    db::{db_provider::PackageRepo, sqlite_db::SqliteDb},
    models::package::{PackageAccessDto, PackageCreateDto, PackagePublicDto},
};
use anyhow::{Ok, Result};
use async_trait::async_trait;

#[async_trait]
impl PackageRepo for SqliteDb {
    // insert a new package
    async fn insert_package(&self, package: PackageCreateDto) -> Result<PackagePublicDto> {
        let sql = "INSERT INTO packages (version, code,package_id) VALUES (?,?,?);";
        let row = sqlx::query_as(sql).bind(package.version).bind(package.code).bind(package.package_id).fetch_one(&self.pool).await?;
        Ok(row)
    }
    // gets package by PACKAGE_ID and VERSION
    async fn get_package(&self, data: PackageAccessDto) -> Result<PackagePublicDto> {
        let sql = "SELECT * FROM packages WHERE package_id = ? AND version = ?;";
        let row = sqlx::query_as(sql).bind(data.package_id).bind(data.version).fetch_one(&self.pool).await?;
        Ok(row)
    }
    // updates the package CODE by PACKAGE_ID and VERSION
    async fn update_package(&self, package: PackageCreateDto) -> Result<PackagePublicDto> {
        let sql = "UPDATE packages SET code = ? WHERE version = ? AND package_id = ?;";
        let row = sqlx::query_as(sql).bind(package.code).bind(package.version).bind(package.package_id).fetch_one(&self.pool).await?;
        Ok(row)
    }
    // get latest package by PACKAGE_ID
    async fn get_latest_package(&self, id: i32) -> Result<PackagePublicDto> {
        let sql = "SELECT * FROM packages WHERE package_id = ? ORDER BY created_at DESC LIMIT 1;";
        let row: PackagePublicDto = sqlx::query_as(sql).bind(id).fetch_one(&self.pool).await?;
        Ok(row)
    }
    // delete all packages by PACKAGE_ID
    async fn delete_package(&self, package_id: i32) -> Result<()> {
        let sql = "DELETE FROM packages WHERE package_id = ?;";
        let _row = sqlx::query(sql)
            .bind(package_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    // delete package by NAME and VERSION
    async fn delete_package_version(&self, package: PackageAccessDto) -> Result<()> {
        let sql = "DELETE FROM packages WHERE package_id = ? AND version = ?;";
        let _row = sqlx::query(sql)
            .bind(package.package_id)
            .bind(package.version)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

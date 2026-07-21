use async_trait::async_trait;
use anyhow::{Ok, Result};

use crate::{db::{db_provider::PackageFileRepo, sqlite_db::SqliteDb}, models::package_file::package_file::PackageFile};


#[async_trait]
impl PackageFileRepo for SqliteDb {
    async fn get_package_file_by_hash(&self, file_hash: String) -> Result<Option<PackageFile>>{
        let sql = "SELECT * FROM package_file WHERE file_hash = ?;";
        let file = sqlx::query_as::<_,PackageFile>(sql).bind(file_hash).fetch_optional(&self.pool).await?;
        Ok(file)
    }
    async fn new_file(&self, file: PackageFile) -> Result<PackageFile>{
        let sql = "INSERT INTO pcakage_file (file_hash, code) VALUES (?,?) RETURNING *;";
        let file = sqlx::query_as::<_,PackageFile>(sql).bind(file.file_hash).bind(file.code).fetch_one(&self.pool).await?;
        Ok(file)
    }
}
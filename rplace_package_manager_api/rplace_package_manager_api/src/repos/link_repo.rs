use async_trait::async_trait;
use anyhow::{Ok, Result};

use crate::{db::{db_provider::LinkRepo, sqlite_db::SqliteDb}, models::link::link::{Link, LinkCreateDto}};

#[async_trait]
impl LinkRepo for SqliteDb {
    async fn get_link_by_package_version_id_and_file_path(&self, package_version_id: i32, file_path: String) -> Result<Link>{
        let package_version_id = package_version_id;
        let file_path = file_path;
        let sql = "SELECT * FROM links WHERE package_version_id = ? AND file_path = ?;";
        let row = sqlx::query_as::<_,Link>(sql).bind(package_version_id).bind(file_path).fetch_one(&self.pool).await?;
        Ok(row)
    }
    async fn new_link(&self, link: LinkCreateDto) -> Result<Link>{
        let sql = "INSERT INTO links (package_version_id, file_hash,file_path) VALUES (?,?,?) RETURNING *;";
        let row = sqlx::query_as::<_,Link>(sql).bind(link.package_version_id).bind(link.file_hash).bind(link.file_path).fetch_one(&self.pool).await?;
        Ok(row)
    }
    // todo: test
    async fn update_link_hash(&self, 
        version_header_id: i32, 
        file_path: String, 
        new_file_hash: String
    ) -> Result<Link>{
        let sql = "UPDATE links SET file_hash = (?) WHERE file_path = (?) AND package_version_id = (?) RETURNING *;";
        let row = sqlx::query_as::<_,Link>(sql).bind(new_file_hash).bind(file_path).bind(version_header_id).fetch_one(&self.pool).await?;
        Ok(row)
    }
}
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct PackageFile{
    pub file_hash: String,
    pub code: String,
}
#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct PackageFileCreateDto {
    pub registry_id: i32,
    pub version_header_id: i32,
    pub code: String,
    pub path: String,
}
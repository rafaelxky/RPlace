use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct Link{
    pub package_version_id: i32,
    pub file_hash: String,
    pub file_path: String,
}
#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct LinkCreateDto{
    pub package_version_id: i32,
    pub file_hash: String,
    pub file_path: String,
}
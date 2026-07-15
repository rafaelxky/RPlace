use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct PackageFile{
    pub file_hash: String,
    pub code: String,
    pub package_id: i32,
}
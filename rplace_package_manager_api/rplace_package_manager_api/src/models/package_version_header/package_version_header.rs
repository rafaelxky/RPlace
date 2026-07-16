use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow)]
pub struct PackageVersionHeader{
    pub id: i32,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub package_id: i32,
}
#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct PackageVersionHeaderCreateDto{
    pub package_name: String,
    pub version: String,
}
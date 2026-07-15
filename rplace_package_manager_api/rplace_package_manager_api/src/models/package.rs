use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow)]
pub struct PackageAccessDto{
    pub package_id: i32,
    pub version: String,
}
#[derive(Debug,Clone,FromRow)]
pub struct PackagePublicDto {
    pub version: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub package_id: i32,
}
#[derive(Debug,Clone,FromRow)]
pub struct PackageCreateDto {
    pub package_id: i32,
    pub version: String,
    pub code: String,
}
#[derive(Debug,Clone,FromRow)]
pub struct Package{
    pub version: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub package_id: i32,
}

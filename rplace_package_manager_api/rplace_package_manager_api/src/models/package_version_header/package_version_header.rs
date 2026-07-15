use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow)]
pub struct PackageVersionHeader{
    pub id: i32,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub package_id: i32,
}
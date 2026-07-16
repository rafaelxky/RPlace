use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;
use serde::{Serialize,Deserialize};

#[derive(Debug,Clone,FromRow)]
pub struct PackageRegistry {
    pub id: i32,
    pub package_name: String,
    pub created_at: DateTime<Utc>,
    pub creator_id: i32,
} 

#[derive(Debug,Clone,FromRow,Serialize, Deserialize)]
pub struct PackageRegistryCreateDto{
    pub name: String,
}

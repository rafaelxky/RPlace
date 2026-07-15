use chrono::{DateTime, Utc};

pub struct PackageAccessDto{
    pub name: String,
    pub version: String,
}
pub struct PackagePublicDto {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub creator_id: i32,
}
pub struct PackageCreateDto {
    pub name: String,
    pub version: String,
    pub code: String,
}
pub struct Package {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub creator_id: i32,
} 

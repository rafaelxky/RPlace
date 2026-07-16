use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow)]
pub struct Link{
    pub package_version_id: i32,
    pub file_hash: String,
    pub file_path: String,
}
#[derive(Debug,Clone,FromRow)]
pub struct LinkCreateDto{
    pub package_version_id: i32,
    pub file_hash: String,
    pub file_path: String,
}
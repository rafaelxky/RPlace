use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow)]
pub struct PackageRegistry {
    pub id: i32,
    pub name: String,
    pub creator_id: i32,
} 
#[derive(Debug,Clone,FromRow)]
pub struct PackageRegistryPublicDto {
    pub id: i32,
    pub name: String,
    pub creator_id: i32,
} 
#[derive(Debug,Clone,FromRow)]
pub struct PackageRegistryCreateDto{
    pub name: String,
    pub creator_id: i32,
}
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow)]
pub struct UserCreateDto{
    pub name: String,
    pub email: String,
    pub password_hash: String,
}
#[derive(Debug,Clone,FromRow)]
pub struct UserPublicDto{
    pub name: String,
    pub id: i32,
}
#[derive(Debug,Clone,FromRow)]
pub struct User{
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password_hash: String,
}
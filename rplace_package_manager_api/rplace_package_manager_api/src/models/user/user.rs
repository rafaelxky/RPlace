use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct UserCreateDto{
    pub name: String,
    pub email: String,
    pub password: String,
}
#[derive(Debug,Clone,FromRow, Serialize, Deserialize)]
pub struct HashedUser{
    pub name: String,
    pub email: String,
    pub password_hash: String,
} 
impl HashedUser {
    pub fn from_create(user: UserCreateDto, hash: String) -> Self{
        Self { name: user.name, email: user.email, password_hash: hash}
    }
}
#[derive(Debug,Clone,FromRow,Serialize,Deserialize)]
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
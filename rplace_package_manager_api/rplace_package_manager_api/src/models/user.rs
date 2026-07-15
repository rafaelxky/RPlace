pub struct UserCreateDto{
    pub name: String,
    pub email: String,
    pub password_hash: String,
}
pub struct UserPublicDto{
    pub name: String,
    pub id: i32,
}
pub struct User{
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password_hash: String,
}
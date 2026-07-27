use serde::Deserialize;
use chrono::{DateTime, Utc};

#[derive(Debug,Clone,Deserialize)]
pub struct CreatedPackageResponse{
    id: i32,
    name: String,
    created_at: DateTime<Utc>,
    creator_id: i32,
}
#[derive(Debug,Clone,Deserialize)]
pub struct CreatedVersionResponse{
    id: i32,
    version: String,
    created_at: DateTime<Utc>,
    package_id: i32,
}
#[derive(Debug,Clone,Deserialize)]
pub struct UploadedFileResponse{
    path: String,
    file_hash: String,
}
#[derive(Debug,Clone,Deserialize)]
pub struct LogginResponse{
    token: String,
}
#[derive(Debug,Clone,Deserialize)]
pub struct CreatedUserResponse{
    id: i32,
    name: String,
}
#[derive(Debug,Clone,Deserialize)]
pub struct ResponsePackageData{
    repo_id: i32,
    version: String,
    header_id: i32,
    file_hash: String,
    file_path: String,
    code: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct ResponseGetPackageFile{
    header_id: i32,
    file_path: String,
    file_hash: String,
    code: String,
}
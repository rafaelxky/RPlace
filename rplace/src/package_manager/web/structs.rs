use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct ErrorResponse{
    pub message: String,
    pub err: String,
}
#[derive(Debug,Clone,Deserialize)]
pub struct CreatedPackageResponse{
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub creator_id: i32,
}
#[derive(Debug,Clone,Deserialize)]
pub struct CreatedVersionResponse{
    pub id: i32,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub package_id: i32,
}
#[derive(Debug,Clone,Deserialize)]
pub struct UploadedFileResponse{
    pub path: String,
    pub file_hash: String,
}
#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct LogginResponse{
    pub token: String,
}
#[derive(Debug,Clone,Deserialize)]
pub struct CreatedUserResponse{
    pub id: i32,
    pub name: String,
}
#[derive(Debug,Clone,Deserialize)]
pub struct ResponsePackageData{
    pub repo_id: i32,
    pub version: String,
    pub header_id: i32,
    pub file_hash: String,
    pub file_path: String,
    pub code: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct ResponseGetPackageFile{
    pub header_id: i32,
    pub file_path: String,
    pub file_hash: String,
    pub code: String,
}

#[derive(Debug,Clone,Deserialize)]
pub struct InitialPackageData{
    pub package_id: i32,
    pub version_id: i32,
}
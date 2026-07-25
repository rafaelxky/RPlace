use anyhow::{Ok, Result};
use reqwest::{Client, header};
use serde_json::json;

use crate::package_manager::web::{URI_BASE, structs::{CreatedPackageResponse, CreatedVersionResponse, UploadedFileResponse}};

pub const CREATE_PACKAGE_URI: &str = "/package";
pub const CREATE_VERSION_URI: &str = "/package/version";
pub const UPLOAD_FILE_URI: &str = "/file";

pub async fn create_new_package(name: &str, token: &str) -> Result<CreatedPackageResponse>{
    let uri = format!("{}{}", URI_BASE,CREATE_PACKAGE_URI);
    let client = Client::new();

    let response = client
    .post(uri)
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .json(&json!(
        {
            "name": name
        }
    )).send().await?;

    let body: CreatedPackageResponse = response.json().await?;

    Ok(body)

} 

pub async fn create_new_version(package_name: &str, version: &str, token: &str) -> Result<CreatedVersionResponse>{
     let uri = format!("{}{}", URI_BASE,CREATE_VERSION_URI);
    let client = Client::new();

    let response = client
    .post(uri)
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .json(&json!(
        {
            "package_name": package_name,
            "version": version
        }
    )).send().await?;

    let body: CreatedVersionResponse = response.json().await?;

    Ok(body)
}

pub async fn upload_file(
    registry_id: i32, 
    version_header_id: i32, 
    code: &str, 
    path: &str,
    token: &str
) -> Result<UploadedFileResponse>{
     let uri = format!("{}{}", URI_BASE,UPLOAD_FILE_URI);
    let client = Client::new();

    let response = client
    .post(uri)
    .header(header::AUTHORIZATION, format!("Bearer {}", token))
    .json(&json!(
        {
            "registry_id": registry_id,
            "version_header_id": version_header_id,
            "code": code,
            "path": path,
        }
    )).send().await?;

    let body: UploadedFileResponse = response.json().await?;
    Ok(body)
}
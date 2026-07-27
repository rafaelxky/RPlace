use anyhow::{Ok, Result};
use reqwest::Client;

use crate::package_manager::web::{URI_BASE, structs::{ResponseGetPackageFile, ResponsePackageData}};

pub const INITIAL_PACKAGE_NO_VERSION_URI: &str = "/package/";
pub const INITIAL_PACKAGE_URI: &str = "/package/";
pub const GET_PACKAGE_FILE_URI: &str = "/package/fetch_file/";

pub async fn get_initial_package(
    package_name: &str,
    package_version: Option<&str>,
) -> Result<ResponsePackageData> {
    let res = match package_version {
        Some(v) => {
            get_initial_package_version(package_name, v).await
        },
        None => {
            get_initial_package_no_version(package_name).await
        }
    }?;
    Ok(res)
}
pub async fn get_initial_package_no_version(package_name: &str) -> Result<ResponsePackageData>{
    let mut uri = format!("{}{}", URI_BASE, INITIAL_PACKAGE_NO_VERSION_URI);
    uri.push_str(package_name);
    let client = Client::new();

    let response = client
        .get(uri)
        .send()
        .await?;

    let body: ResponsePackageData = response.json().await?;

    Ok(body)
}
pub async fn get_initial_package_version(package_name: &str, package_version: &str) -> Result<ResponsePackageData>{
    let mut uri = format!("{}{}", URI_BASE, INITIAL_PACKAGE_URI);
    uri.push_str(package_name);
    uri.push_str("/");
    uri.push_str(package_version);
    let client = Client::new();

    let response = client
        .get(uri)
        .send()
        .await?;

    let body: ResponsePackageData = response.json().await?;

    Ok(body)
}

pub async fn get_package_file(version_header_id: i32, path: &str) -> Result<ResponseGetPackageFile>{
    let mut uri = format!("{}{}", URI_BASE, GET_PACKAGE_FILE_URI);
    uri.push_str(&format!("{}",version_header_id));
    uri.push_str("/");
    uri.push_str(path);
    let client = Client::new();

    let response = client
        .get(uri)
        .send()
        .await?;

    let body: ResponseGetPackageFile = response.json().await?;

    Ok(body)
}
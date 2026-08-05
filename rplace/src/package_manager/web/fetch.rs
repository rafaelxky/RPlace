use anyhow::{Ok, Result};
use reqwest::Client;

use crate::package_manager::web::{structs::{ResponseGetPackageFile, ResponsePackageData}};

pub const INITIAL_PACKAGE_NO_VERSION_URI: &str = "/package/";
pub const INITIAL_PACKAGE_URI: &str = "/package/";
pub const GET_PACKAGE_FILE_URI: &str = "/package/fetch_file/";

// gets the rplace.toml file from web
pub async fn get_initial_package(
    package_source: &str,
    package_name: &str,
    package_version: Option<&str>,
) -> Result<ResponsePackageData> {
    let res = match package_version {
        Some(v) => {
            get_initial_package_version(package_source,package_name, v).await
        },
        None => {
            get_initial_package_no_version(package_source,package_name).await
        }
    }?;
    Ok(res)
}
// gets the rplace.toml file from web with latest version
pub async fn get_initial_package_no_version(package_source: &str, package_name: &str) -> Result<ResponsePackageData>{
    let mut uri = format!("{}{}", package_source, INITIAL_PACKAGE_NO_VERSION_URI);
    uri.push_str(package_name);
    let client = Client::new();

    let response = client
        .get(uri)
        .send()
        .await?;

    let body: ResponsePackageData = response.json().await?;

    Ok(body)
}
pub async fn get_initial_package_version(package_source: &str,package_name: &str, package_version: &str) -> Result<ResponsePackageData>{
    let mut uri = format!("{}{}", package_source, INITIAL_PACKAGE_URI);
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

// gets a package file
pub async fn get_package_file(package_source: &str,version_header_id: i32, path: &str) -> Result<ResponseGetPackageFile>{
    let mut uri = format!("{}{}", package_source, GET_PACKAGE_FILE_URI);
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
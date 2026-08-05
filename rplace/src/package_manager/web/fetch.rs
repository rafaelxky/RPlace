use anyhow::{Result, anyhow};
use reqwest::Client;

use crate::package_manager::web::structs::{
    InitialPackageData, ResponseGetPackageFile, ResponsePackageData,
};

pub const INITIAL_PACKAGE_NO_VERSION_URI: &str = "/package/";
pub const INITIAL_PACKAGE_URI: &str = "/package/";
pub const GET_PACKAGE_FILE_URI: &str = "/package/fetch_file/";
pub const INITIAL_PACKAGE_FETCH_DATA: &str = "/package/data/";

// gets the rplace.toml file from web
pub async fn get_initial_package(
    package_source: &str,
    package_name: &str,
    package_version: Option<&str>,
) -> Result<ResponsePackageData> {
    let res = match package_version {
        Some(v) => get_initial_package_version(package_source, package_name, v).await,
        None => get_initial_package_no_version(package_source, package_name).await,
    }?;
    Ok(res)
}
// gets the rplace.toml file from web with latest version
pub async fn get_initial_package_no_version(
    package_source: &str,
    package_name: &str,
) -> Result<ResponsePackageData> {
    let uri = format!("{}{}{}", package_source, INITIAL_PACKAGE_NO_VERSION_URI,package_name);
    let client = Client::new();

    let response = client.get(uri).send().await?;
    if !response.status().is_success() {
        let body: String = response.text().await?;
        return Err(anyhow!(body));
    }

    let body: ResponsePackageData = response.json().await?;

    Ok(body)
}
pub async fn get_initial_package_version(
    package_source: &str,
    package_name: &str,
    package_version: &str,
) -> Result<ResponsePackageData> {
    let uri = format!("{}{}{}/{}", package_source, INITIAL_PACKAGE_URI, package_name,package_version);
    let client = Client::new();

    let response = client.get(uri).send().await?;
    if !response.status().is_success() {
        let body: String = response.text().await?;
        return Err(anyhow!(body));
    }

    let body: ResponsePackageData = response.json().await?;

    Ok(body)
}

// gets a package file
pub async fn get_package_file(
    package_source: &str,
    version_header_id: i32,
    path: &str,
) -> Result<ResponseGetPackageFile> {
    let mut uri = format!("{}{}", package_source, GET_PACKAGE_FILE_URI);
    uri.push_str(&format!("{}", version_header_id));
    uri.push_str("/");
    uri.push_str(path);
    let client = Client::new();

    let response = client.get(uri).send().await?;
    if !response.status().is_success() {
        let body: String = response.text().await?;
        return Err(anyhow!(body));
    }

    let body: ResponseGetPackageFile = response.json().await?;

    Ok(body)
}

pub async fn get_initial_data(
    package_source: &str,
    package_name: &str,
    version_name: &str,
) -> Result<InitialPackageData> {
    let uri = format!(
        "{}{}{}/{}",
        package_source, INITIAL_PACKAGE_FETCH_DATA, package_name, version_name
    );
    let client = Client::new();

    let response = client.get(uri).send().await?;
    if !response.status().is_success() {
        let body: String = response.text().await?;
        return Err(anyhow!(body));
    }

    let body: InitialPackageData = response.json().await?;

    Ok(body)
}

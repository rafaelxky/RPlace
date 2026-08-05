use anyhow::{Result};
use reqwest::{Client, StatusCode};
use serde_json::json;
use anyhow::anyhow;

use crate::package_manager::web::structs::{ErrorResponse, LogginResponse};

pub const LOGGIN_URI: &str = "/loggin";

pub async fn loggin(package_source: &str, email: &str, password: &str) -> Result<LogginResponse> {
    let uri = format!("{}{}", package_source, LOGGIN_URI);

    let client = Client::new();
    let response = client
        .post(uri)
        .json(&json!(
            {
                "email": email,
                "password": password,
            }
        ))
        .send()
        .await?;

    if !response.status().is_success() {
        let body: String = response.text().await?;
        return Err(anyhow!(body));
    }

    let body: LogginResponse = response.json().await?;

    Ok(body)
}

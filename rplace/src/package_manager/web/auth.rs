use anyhow::{Ok, Result};
use reqwest::{Client};
use serde_json::json;

use crate::package_manager::web::{structs::LogginResponse};


pub const LOGGIN_URI: &str = "/loggin";

pub async fn loggin(package_source: &str,email: &str, password: &str) -> Result<LogginResponse> {
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

    let body: LogginResponse = response.json().await?;

    Ok(body)
}

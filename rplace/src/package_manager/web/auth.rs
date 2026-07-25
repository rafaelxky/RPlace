use anyhow::{Ok, Result};
use reqwest::{Client};
use serde_json::json;

use crate::package_manager::web::{structs::LogginResponse, URI_BASE};


pub const LOGGIN_URI: &str = "/loggin";

pub async fn loggin(email: &str, password: &str) -> Result<LogginResponse> {
    let uri = format!("{}{}", URI_BASE, LOGGIN_URI);

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

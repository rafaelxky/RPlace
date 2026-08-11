use anyhow::{Result};
use reqwest::{Client};
use serde_json::json;
use anyhow::anyhow;

use crate::package_manager::web::structs::{LoginResponse};

pub const LOGIN_URI: &str = "/loggin";

pub async fn login(package_source: &str, email: &str, password: &str) -> Result<LoginResponse> {
    let uri = format!("{}{}", package_source, LOGIN_URI);

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

    let body: LoginResponse = response.json().await?;

    Ok(body)
}

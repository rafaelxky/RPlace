use anyhow::{Ok, Result};
use reqwest::Client;
use serde_json::json;

use crate::package_manager::web::{structs::CreatedUserResponse};

const CREATE_USER_URI: &str = "/user";

pub async fn create_user(package_source: &str,name: &str, email: &str, password: &str) -> Result<CreatedUserResponse>{
    let uri =  format!("{}{}", package_source,CREATE_USER_URI);
    let client = Client::new();

    let response = client.post(uri)
    .json(&json!({
        "name": name,
        "email": email,
        "password": password
    }
    )).send().await?;

    let body: CreatedUserResponse = response.json().await?;

    Ok(body)
}
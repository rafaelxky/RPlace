use anyhow::Result;
use jsonwebtoken::{DecodingKey, Validation, decode, TokenData};


use crate::models::loggin::loggin::JwtClaims;

pub fn can_access(token: &str) -> Result<JwtClaims> {
    let token = token.strip_prefix("Bearer ").unwrap_or(token);
    let jwt_secret = std::env::var("JWT_SECRET");
    let jwt_secret = match jwt_secret {
        Ok(s) => s,
        Err(e) => {
            println!("JWT_SECRET not set!");
            return Err(e.into());
        }
    };
    let jwt_secret = jwt_secret.as_bytes();
    let data: TokenData<JwtClaims> = decode::<JwtClaims>(token, &DecodingKey::from_secret(jwt_secret), &Validation::default())?;
    return Ok(data.claims);
}
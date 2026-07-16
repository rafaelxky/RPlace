use argon2::{Argon2, PasswordHash,PasswordVerifier};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::{post}};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, jws::encode};
use serde_json::json;

use crate::models::{
    app_state::AppState,
    loggin::loggin::{JwtClaims, LogginRequest},
};

pub fn routes() -> Router<AppState> {
    Router::new()
    .route("/loggin", post(loggin))
}

// /loggin GET
// checks email and pasword hash
// if ok (user exists and password and name match), create and return a jwt claim
/*  
input: 
{
    "username": string,
    "password": string
}

returns:
{
    "token": string
}
*/
pub async fn loggin(
    State(state): State<AppState>,
    Json(loggin): Json<LogginRequest>,
) -> (StatusCode, impl IntoResponse) {
    let request = loggin;
    let user = state.db_provider.get_user_by_email(request.email).await;
    let user = match user {
        Ok(u) => u,
        Err(_e) => {
            println!("didn't find user");
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!(
                    {
                        "message": "wrong password or email",
                    }
                )),
            );
        }
    };

    let parsed_hash = PasswordHash::new(&user.password_hash);
    let parsed_hash = match parsed_hash {
        Ok(h) => h,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "could not hash password",
                    "err": &e.to_string()
                })),
            );
        }
    };

    let result = Argon2::default().verify_password(request.password.as_bytes(), &parsed_hash);

    match result {
        Ok(()) => (),
        Err(e) => {
            println!("hash: {}", user.password_hash);
            println!("parsed: {}", parsed_hash);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "message": "wrong password or email",
                    "err": &e.to_string()
                })),
            );
        }
    }

    let claims = JwtClaims {
        user_id: user.id,
        expiration_date: (Utc::now() + Duration::days(7)).timestamp() as usize,
    };

    let jwt_secret = std::env::var("JWT_SECRET");
    let jwt_secret = match &jwt_secret {
        Ok(s) => s.as_bytes(),
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "jwt secret not set",
                "err": &e.to_string()
            })));
        },
    };

    let token = encode(
        &Header::default(),
        Some(&claims),
        &EncodingKey::from_secret(jwt_secret),
    );

    let token = match token {
        Ok(t) => t,
        Err(e) => {
            return(StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "could not generate jwt token",
                "err": &e.to_string()
            })));
        },
    };

    return (StatusCode::OK,Json(json!({
        "token": token
    })));
}

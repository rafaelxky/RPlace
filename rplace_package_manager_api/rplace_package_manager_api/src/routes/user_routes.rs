use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use rand_core::OsRng;
use serde_json::json;

use crate::models::{app_state::AppState, user::user::{HashedUser, UserCreateDto}};

pub fn routes() -> Router<AppState> {
    Router::new().route("/user", post(new_user))
}

async fn new_user(
    State(state): State<AppState>,
    Json(body): Json<UserCreateDto>,
) -> impl IntoResponse {
    let new_user = body;

    let salt = SaltString::generate(&mut OsRng);

    let password_hash = Argon2::default()
        .hash_password(new_user.password_hash.as_bytes(), &salt);

    let password_hash = match password_hash {
        Ok(h) => h.to_string(),
        Err(e) => {
            return Json(json!({
                "message": "could not hash password",
                "e": &e.to_string(),
            }));
        }
    };

    let new_user = HashedUser::from_create(new_user, password_hash);

    let user = state.db_provider.new_user(new_user).await;
    let user = match user {
        Ok(u) => u,
        Err(e) => {
            return Json(json!({
                "message": "could not create new user",
                "err": &e.to_string()
            }));
        }
    };
    return Json(json!(
        {
            "id": &user.id,
            "name": &user.name
        }
    ));
}

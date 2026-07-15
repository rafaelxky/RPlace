use axum::{Router};

use crate::models::app_state::AppState;

pub fn routes() -> Router<AppState>{
    Router::new()
}

// packages/name/version
async fn get_package() -> String{
    todo!()
}
async fn new_package(){
    todo!()
}
async fn update_package(){
    todo!()
}
async fn delete_package(){
    todo!()
}
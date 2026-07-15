use axum::Router;

pub mod package_routes;

pub fn router() -> Router {
    Router::new().merge(package_routes::routes())
}
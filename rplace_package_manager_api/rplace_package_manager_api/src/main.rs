use axum::Router;

pub mod routes;
pub mod service;
pub mod models;

#[tokio::main]
async fn main() {
    println!("Starting server...");

    let app = Router::new().merge(routes::router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

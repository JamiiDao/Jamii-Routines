use axum::{Router, http::Method, routing::get};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    let origins = [
        "http://localhost:5173".parse().unwrap(),
        "http://127.0.0.1:5173".parse().unwrap(),
    ];

    #[cfg(debug_assertions)]
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST]);
    #[cfg(not(debug_assertions))]
    let cors = CorsLayer::new();

    let app = Router::new().route("/", get(root)).layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, world"
}

use axum::{Router, extract::State, http::Method, routing::get};
use tower_http::cors::CorsLayer;

mod auth;
pub use auth::*;

mod storage;
pub use storage::*;

mod errors;
pub use errors::*;

#[tokio::main]
async fn main() {
    let app_db_path = "routines.db";
    let app_db = AppDb::init(app_db_path).await.unwrap();
    app_db.create_tables_if_missing().await.unwrap();

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

    let app = Router::new()
        .route("/", get(root))
        .with_state(app_db)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn root(State(state): State<AppDb>) -> &'static str {
    // example query (optional)
    let _ = state.db.acquire().await;

    "Hello, world"
}

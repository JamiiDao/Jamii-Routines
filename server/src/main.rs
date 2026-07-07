#[cfg(debug_assertions)]
use axum::http::{HeaderValue, header::CONTENT_TYPE};
use axum::{
    Router,
    extract::State,
    http::Method,
    routing::{get, post},
};

use tower_http::cors::CorsLayer;

mod app_routes;
pub use app_routes::*;

mod auth;
pub use auth::*;

mod storage;
pub use storage::*;

mod errors;
pub use errors::*;

mod handlers;
pub use handlers::*;

mod parse_secrets;
pub use parse_secrets::*;

#[tokio::main]
async fn main() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    Secrets::asserts();

    EmailService::init_smtps().await;

    let app_db_path = "routines.db";
    let app_db = AppDb::init(app_db_path).await.unwrap();
    app_db.create_tables_if_missing().await.unwrap();

    CookieAuthProcessor::cleanup(app_db.db.clone());

    #[cfg(debug_assertions)]
    let cors = CorsLayer::new()
        .allow_origin(["http://127.0.0.1:5173".parse::<HeaderValue>().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true)
        .allow_headers([CONTENT_TYPE]);
    #[cfg(not(debug_assertions))]
    let cors = CorsLayer::new();

    let app = Router::new()
        .route(AppRoutes::Root.as_str(), get(root))
        .route(AppRoutes::Login.as_str(), post(RouteHandler::process_login))
        .route(
            AppRoutes::SignUp.as_str(),
            post(RouteHandler::process_signup),
        )
        .route(
            AppRoutes::ResendCode.as_str(),
            post(RouteHandler::process_resend_code),
        )
        .route(
            AppRoutes::VerifyCode.as_str(),
            post(RouteHandler::verify_code),
        )
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

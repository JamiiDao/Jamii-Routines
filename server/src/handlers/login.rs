use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{AppDb, AppRoutes, HttpErrorWrapper, RouteHandler};

impl RouteHandler {
    pub async fn process_login(
        State(state): State<AppDb>,
        jar: CookieJar,
        credentials: Json<LoginData>,
    ) -> Result<(StatusCode, CookieJar, Json<LoginResponse>), HttpErrorWrapper> {
        let email = credentials.email.trim().to_string();
        sqlx::query(
            r#"
                SELECT
                    name
                FROM users
                WHERE name = ?
                "#,
        )
        .bind(email.as_str())
        .fetch_optional(&state.db)
        .await?
        .ok_or(
            HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("Invalid credentials or try registering!"),
        )?;

        Ok((
            StatusCode::SEE_OTHER,
            jar,
            LoginResponse::new(email.as_str())
                .set_path(AppRoutes::VerifyCode.as_str())
                .build(),
        ))
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub email: String,
    pub path: String,
}

impl LoginResponse {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
            path: AppRoutes::Dashboard.as_str().to_string(),
        }
    }

    pub fn set_path(mut self, path: &str) -> Self {
        self.path = path.to_string();

        self
    }

    pub fn build(self) -> Json<Self> {
        Json(self)
    }
}

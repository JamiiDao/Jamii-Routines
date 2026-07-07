use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use sqlx::Row;
use tai64::Tai64N;

use crate::{AppDb, CookieAuthProcessor, HttpErrorWrapper, RedirectData, RouteHandler};

impl RouteHandler {
    pub async fn verify_code(
        State(state): State<AppDb>,
        jar: CookieJar,
        credentials: Json<EmailAuthCodeData>,
    ) -> Result<(StatusCode, CookieJar, Json<RedirectData>), HttpErrorWrapper> {
        let email = credentials.email.trim().to_string();

        let row = sqlx::query(
            r#"
                SELECT
                    name,
                    email_auth_code,
                    email_auth_time
                FROM users
                WHERE name = ?
            "#,
        )
        .bind(email.as_str())
        .fetch_one(&state.db)
        .await?;

        let stored_code = row.try_get::<String, _>("email_auth_code")?;
        if stored_code != credentials.code.to_uppercase() {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("Invalid authentication code"));
        }

        let email_auth_time: [u8; 12] = row
            .try_get::<Vec<u8>, _>("email_auth_time")?
            .try_into()
            .inspect_err(|_| {
                tracing::error!("Unable to convert sqlite blob for `email_auth_time` to [u8;12]");
            })
            .or(Err(HttpErrorWrapper::new()))?;

        let email_auth_time: Tai64N = email_auth_time
            .try_into()
            .inspect_err(|error| {
                tracing::error!(
                    "Unable to convert `email_auth_time` [u8;12] to Tai64N. Error: {error:?}."
                );
            })
            .or(Err(HttpErrorWrapper::new()))?;

        if Tai64N::now() >= email_auth_time + Duration::from_mins(10) {
            Err(HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("Authentication code has expired"))
        } else {
            let processed = CookieAuthProcessor::set_login_headers(&state.db, jar, &email).await?;

            Ok((StatusCode::OK, processed.0, processed.1))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailAuthCodeData {
    email: String,
    code: String,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub mode: String,
}

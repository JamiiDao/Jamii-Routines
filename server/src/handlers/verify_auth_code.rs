use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode, response::Redirect};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use sqlx::Row;
use tai64::Tai64N;

use crate::{AppDb, CookieAuthProcessor, HttpErrorWrapper, RouteHandler};

impl RouteHandler {
    pub async fn verify_code(
        State(state): State<AppDb>,
        jar: CookieJar,
        credentials: Json<EmailAuthCodeData>,
    ) -> Result<(CookieJar, Redirect), HttpErrorWrapper> {
        let row = sqlx::query(
            r#"
                SELECT
                    name,
                    email_auth_code,
                    email_auth_time,
                FROM users
                WHERE name = ?
            "#,
        )
        .bind(credentials.email.as_str())
        .fetch_one(&state.db)
        .await?;

        let stored_code = row.try_get::<String, _>("email_auth_code")?;
        if stored_code != credentials.code {
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

        if (email_auth_time + Duration::from_mins(10)) > Tai64N::now() {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("Authentication code already expired"));
        }

        CookieAuthProcessor::set_login_headers(&state.db, jar, &credentials.email).await
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailAuthCodeData {
    email: String,
    code: String,
}

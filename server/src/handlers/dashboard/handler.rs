use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use sqlx::Row;

use crate::{AppDb, CookieAuthProcessor, HttpErrorWrapper, PasskeyData, RouteHandler};

impl RouteHandler {
    pub async fn dashboard_data(
        State(state): State<AppDb>,
        jar: CookieJar,
    ) -> Result<(StatusCode, Json<DashboardData>), HttpErrorWrapper> {
        let (_, email, _) = CookieAuthProcessor::check_cookies(&state.db, &jar).await?;

        let row = sqlx::query(
            r#"
                SELECT
                    name,
                    passkey,
                    create_time
                FROM users
                WHERE name = ?
                "#,
        )
        .bind(email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| {
            HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("expired session. New login required!")
        })?;

        let create_time = row.try_get::<String, _>("create_time")?;
        let passkey_exists = row
            .try_get::<Option<String>, _>("passkey")?
            .map(|value| {
                let passkey_state = serde_json::from_str::<PasskeyData>(&value)?;

                let passkey_valid = match passkey_state.state.index() {
                    0 => false,
                    _ => true,
                };

                Ok::<_, HttpErrorWrapper>(passkey_valid)
            })
            .transpose()?
            .unwrap_or_default();

        let data = DashboardData {
            create_time,
            passkey_exists,
        };

        Ok((StatusCode::OK, Json(data)))
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub create_time: String,
    pub passkey_exists: bool,
}

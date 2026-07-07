use std::time::Duration;

use axum::{Json, http::StatusCode};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use random_detached::{Generator, RandomBytes};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, query};
use tai64::Tai64N;

use crate::{AppRoutes, ErrorResponse, HttpErrorWrapper};

pub struct CookieAuthProcessor;

impl CookieAuthProcessor {
    pub async fn check_cookies(
        db: &Pool<Sqlite>,
        jar: CookieJar,
        email: &str,
    ) -> Result<(StatusCode, CookieJar), HttpErrorWrapper> {
        let cookie_exists = jar
            .get("session")
            .ok_or_else(|| {
                HttpErrorWrapper::new()
                    .status_code(StatusCode::UNAUTHORIZED)
                    .message("expired session. New login required!")
            })?
            .clone();

        let cookie_value = cookie_exists.value();

        let row = sqlx::query(
            r#"
                        SELECT
                            user_name,
                            created_at
                        FROM sessions
                        WHERE token = ?
                        "#,
        )
        .bind(cookie_value)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| {
            dbg!("COOKIE FOUND IN HEADER BUT NOT FOUND IN STORE");

            HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("expired session. New login required!")
        })?;

        let user_name = row.try_get::<String, _>("user_name")?;

        if user_name != email {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .message("expired session. New login required!"));
        }

        let created_at: [u8; 12] = row
            .try_get::<Vec<u8>, _>("created_at")?
            .try_into()
            .inspect_err(|_| {
                tracing::error!("Unable to convert sqlite blob for `created_at` to [u8;12]");
            })
            .or(Err(HttpErrorWrapper::new()))?;

        let created_at: Tai64N = created_at
            .try_into()
            .inspect_err(|error| {
                tracing::error!(
                    "Unable to convert `created_at` [u8;12] to Tai64N. Error: {error:?}."
                );
            })
            .or(Err(HttpErrorWrapper::new()))?;

        if Tai64N::now() >= created_at + Duration::from_hours(24) {
            let new_jar = jar.remove(Cookie::from("session"));

            sqlx::query(
                r#"
                DELETE FROM sessions
                WHERE token = ?
                "#,
            )
            .bind(cookie_value)
            .execute(db)
            .await?;

            return Ok((StatusCode::UNAUTHORIZED, new_jar));
        }

        Ok((StatusCode::OK, jar))
    }

    pub async fn set_login_headers(
        db: &Pool<Sqlite>,
        jar: CookieJar,
        email: &str,
    ) -> Result<(CookieJar, Json<RedirectData>), HttpErrorWrapper> {
        let session_token_bytes = RandomBytes::<32>::generate().map_err(|error| {
            tracing::error!("set_login_headers. {error:?}");

            HttpErrorWrapper((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    message: "Internal server error".into(),
                }),
            ))
        })?;
        let session_token = faster_hex::hex_string_upper(session_token_bytes.as_slice());
        let now = Tai64N::now();
        query(
            r#"
                INSERT INTO sessions (token, user_name, created_at)
                VALUES (?, ?, ?)
                "#,
        )
        .bind(&session_token)
        .bind(email)
        .bind(now.to_bytes().as_slice())
        .execute(db)
        .await?;

        let cookie = Cookie::build(("session", session_token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(cfg!(not(debug_assertions)))
            .max_age(time::Duration::hours(24))
            .build();

        Ok((
            jar.add(cookie),
            RedirectData::new(AppRoutes::Dashboard.as_str()),
        ))
    }

    pub fn cleanup(db: Pool<Sqlite>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));

            loop {
                interval.tick().await;

                let cutoff = (Tai64N::now() - Duration::from_hours(24)).to_bytes();

                if let Err(error) = query(
                    r#"
                        DELETE FROM sessions
                        WHERE created_at < ?
                        "#,
                )
                .bind(cutoff.as_slice())
                .execute(&db)
                .await
                {
                    tracing::error!("Failed to clean expired sessions: {error}");
                }
            }
        });
    }
}

#[derive(Debug, Deserialize)]
pub struct UserCredentials {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct RedirectData {
    pub path: String,
}

impl RedirectData {
    pub fn new(path: &str) -> Json<Self> {
        Json(Self {
            path: path.to_string(),
        })
    }
}

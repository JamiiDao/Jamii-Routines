use axum::{Json, extract::State, http::StatusCode, response::Redirect};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use random_detached::{Generator, RandomBytes};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, query};

use crate::{AppDb, AppRoutes, ErrorResponse, HttpErrorWrapper};

pub struct CookieAuthProcessor;

impl CookieAuthProcessor {
    pub async fn process(
        State(state): State<AppDb>,
        jar: CookieJar,
        credentials: Json<UserCredentials>,
    ) -> Result<(CookieJar, Redirect), HttpErrorWrapper> {
        if let Some(session) = jar.get("session") {
            let session_exists = session.value();

            let row = sqlx::query(
                r#"
        SELECT users.name
        FROM sessions
        JOIN users ON users.name = sessions.user_name
        WHERE sessions.token = ?
          AND sessions.expires_at > datetime('now')
        "#,
            )
            .bind(session_exists)
            .fetch_optional(&state.db)
            .await?;

            if row.is_some() {
                return Ok((jar, Redirect::to(AppRoutes::Dashboard.as_str())));
            }
        }

        if let Some(email_exists) = credentials.0.email {
            let email = sqlx::query_scalar::<_, String>("SELECT name FROM users WHERE name = ?")
                .bind(email_exists.trim())
                .fetch_optional(&state.db)
                .await?;

            if let Some(email) = email {
                Self::set_login_headers(&state.db, jar, &email).await
            } else {
                Err(HttpErrorWrapper((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        message: "Sign Up first or use another email address".into(),
                    }),
                )))
            }
        } else {
            Err(HttpErrorWrapper((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    message: "Invalid or expired session".into(),
                }),
            )))
        }
    }

    pub async fn set_login_headers(
        db: &Pool<Sqlite>,
        jar: CookieJar,
        email: &str,
    ) -> Result<(CookieJar, Redirect), HttpErrorWrapper> {
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

        query(
            r#"
    INSERT INTO sessions (token, user_name, created_at, expires_at)
    VALUES (?, ?, datetime('now'), datetime('now', '+1 day'))
    "#,
        )
        .bind(&session_token)
        .bind(email)
        .execute(db)
        .await?;

        let cookie = Cookie::build(("session", session_token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(cfg!(not(debug_assertions)))
            .build();

        Ok((jar.add(cookie), Redirect::to(AppRoutes::Dashboard.as_str())))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCredentials {
    pub email: Option<String>,
}

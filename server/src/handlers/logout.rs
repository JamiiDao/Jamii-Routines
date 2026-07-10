use axum::{extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;

use crate::{AppDb, CookieAuthProcessor, HttpErrorWrapper, RouteHandler};

impl RouteHandler {
    pub async fn process_logout(
        State(state): State<AppDb>,
        jar: CookieJar,
    ) -> Result<(StatusCode, CookieJar), HttpErrorWrapper> {
        let (_, _, token) = CookieAuthProcessor::check_cookies(&state.db, &jar).await?;

        sqlx::query(
            r#"
                DELETE FROM sessions
                WHERE token = ?
            "#,
        )
        .bind(token)
        .execute(&state.db)
        .await?;

        let jar = jar.remove("session");

        Ok((StatusCode::OK, jar))
    }
}

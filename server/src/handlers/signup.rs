use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{AppDb, HttpErrorWrapper, RouteHandler};

impl RouteHandler {
    pub async fn process_signup(
        State(state): State<AppDb>,
        credentials: Json<SignupData>,
    ) -> Result<StatusCode, HttpErrorWrapper> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE name = ?)")
            .bind(credentials.email.as_str())
            .fetch_one(&state.db)
            .await?;

        if exists {
            return Err(HttpErrorWrapper::new()
                .status_code(StatusCode::BAD_REQUEST)
                .message(
                    "Unable to create the account! If the email is correct maybe try logging in to check whether your account already exists!",
                ));
        } else {
            Self::send_auth(false, credentials.email.as_str(), &state.db).await?;
        }

        Ok(StatusCode::OK)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupData {
    pub email: String,
}

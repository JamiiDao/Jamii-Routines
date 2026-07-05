use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use random_detached::getrandom;
use serde::Serialize;

pub type RoutinesResult<T> = Result<T, RoutinesError>;

#[derive(Debug, thiserror::Error)]
pub enum RoutinesError {
    #[error("{0}")]
    Storage(#[from] sqlx::Error),
    #[error("Unable to create random number for token")]
    Randomness(String),
}

impl From<getrandom::Error> for RoutinesError {
    fn from(value: getrandom::Error) -> Self {
        Self::Randomness(value.to_string())
    }
}

impl From<sqlx::Error> for HttpErrorWrapper {
    fn from(value: sqlx::Error) -> Self {
        tracing::error!("sqlx error: {value:?}");

        Self((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                message: "Internal Server Error".to_string(),
            }),
        ))
    }
}

pub type ErrorResponseType = (StatusCode, Json<ErrorResponse>);

pub struct HttpErrorWrapper(pub ErrorResponseType);

impl HttpErrorWrapper {
    pub fn new() -> Self {
        Self((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                message: "Internal server error occurred".to_string(),
            }),
        ))
    }

    pub fn status_code(mut self, status_code: StatusCode) -> Self {
        self.0.0 = status_code;

        self
    }

    pub fn message(mut self, message: &str) -> Self {
        self.0.1 = Json(ErrorResponse {
            message: message.to_string(),
        });

        self
    }
}

impl IntoResponse for HttpErrorWrapper {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

impl From<RoutinesError> for HttpErrorWrapper {
    fn from(error: RoutinesError) -> Self {
        tracing::error!("Randomness: {error:?}");

        Self::new()
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

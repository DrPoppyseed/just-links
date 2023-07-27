use async_session::serde_json::json;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug, Clone)]
pub enum Error {
    Cookie(String),
    Pocket(String),
    Internal(String),
    BadRequest(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Cookie(message) => write!(f, "error: {message}"),
            Error::Pocket(message) => write!(f, "error: {message}"),
            Error::Internal(message) => write!(f, "error: {message}"),
            Error::BadRequest(message) => write!(f, "error: {message}"),
        }
    }
}

impl From<pockety::Error> for Error {
    fn from(error: pockety::Error) -> Self {
        Error::Pocket(error.to_string())
    }
}

impl From<async_session::Error> for Error {
    fn from(error: async_session::Error) -> Self {
        Error::Cookie(error.to_string())
    }
}

impl From<async_session::serde_json::Error> for Error {
    fn from(error: async_session::serde_json::Error) -> Self {
        Error::Cookie(error.to_string())
    }
}

impl From<axum::Error> for Error {
    fn from(error: axum::Error) -> Self {
        Error::Internal(error.to_string())
    }
}

impl From<biscuit::errors::Error> for Error {
    fn from(error: biscuit::errors::Error) -> Self {
        Error::Internal(error.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!("{self:#?}");

        let (status, error_message) = match self {
            Error::Cookie(_) | Error::BadRequest(_) => (StatusCode::BAD_REQUEST, "Unauthorized"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

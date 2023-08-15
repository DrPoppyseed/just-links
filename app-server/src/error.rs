use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, Clone)]
pub enum Error {
    Cookie(String),
    Session(String),
    Pocket(String),
    Jwt(String),
    Api(ApiError),
}

#[derive(Debug, Clone)]
pub enum ApiError {
    BadRequest(String),
    InternalServerError(String),
    Unauthorized(String),
}

impl From<pockety::Error> for Error {
    fn from(error: pockety::Error) -> Self {
        Error::Pocket(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Cookie(error.to_string())
    }
}

impl From<axum::Error> for Error {
    fn from(error: axum::Error) -> Self {
        Error::Api(ApiError::InternalServerError(error.to_string()))
    }
}

impl From<biscuit::errors::Error> for Error {
    fn from(error: biscuit::errors::Error) -> Self {
        Error::Api(ApiError::InternalServerError(error.to_string()))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!("{self:#?}");

        let (status, error_message) = match self {
            Error::Cookie(_) | Error::Api(ApiError::BadRequest(_)) => {
                (StatusCode::BAD_REQUEST, "Bad Request")
            }
            Error::Api(ApiError::Unauthorized(_)) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

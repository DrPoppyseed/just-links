#![feature(result_option_inspect)]

use api::auth::OAuthStateParam;
use async_session::MemoryStore;
use axum::{
    extract::FromRef,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use biscuit::jwk::JWK;
use error::Error;
use pockety::Pockety;
use serde::Serialize;

pub mod api;
pub mod error;
pub mod oauth;
pub mod session;

pub static COOKIE_NAME: &str = "ID";

pub type ApiResult<R> = Result<TypedResponse<R>, Error>;

#[derive(Debug, Clone)]
pub struct TypedResponse<B>
where
    B: Serialize,
{
    body: Option<B>,
    headers: Option<HeaderMap>,
    status_code: StatusCode,
}

impl<B> TypedResponse<B>
where
    B: Serialize,
{
    fn new(body: Option<B>) -> Self {
        TypedResponse {
            body,
            ..Default::default()
        }
    }

    fn headers(self, headers: Option<HeaderMap>) -> Self {
        Self { headers, ..self }
    }

    fn status_code(self, status_code: StatusCode) -> Self {
        Self {
            status_code,
            ..self
        }
    }
}

impl<B> Default for TypedResponse<B>
where
    B: Serialize,
{
    fn default() -> Self {
        Self {
            body: None,
            headers: None,
            status_code: StatusCode::OK,
        }
    }
}

impl<B> IntoResponse for TypedResponse<B>
where
    B: Serialize,
{
    fn into_response(self) -> Response {
        let mut response = Json(self.body).into_response();
        if let Some(headers) = self.headers {
            *response.headers_mut() = headers;
        }
        *response.status_mut() = self.status_code;
        response
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub jws_signing_secret: String,
    pub jwe_encryption_key: JWK<OAuthStateParam>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub pockety: Pockety,
    pub store: MemoryStore,
    pub config: Config,
}

impl FromRef<AppState> for Pockety {
    fn from_ref(state: &AppState) -> Self {
        state.pockety.clone()
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

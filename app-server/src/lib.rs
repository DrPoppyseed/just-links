use std::sync::Arc;

use axum::{
    extract::FromRef,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bb8_redis::RedisConnectionManager;
use biscuit::{jwk::JWK, jws::Secret};
use error::Error;
use oauth::OAuthState;
use pockety::{Pockety, RateLimits as PocketyRateLimits};
use serde::{Deserialize, Serialize};

pub mod api;
pub mod db;
pub mod domain;
pub mod error;
pub mod oauth;
pub mod session;

pub static SESSION_ID_COOKIE_NAME: &str = "ID";

pub type ApiResult<R> = Result<TypedResponse<R>, Error>;
pub type Store = Arc<sqlx::Pool<sqlx::Postgres>>;
pub type Cache = Arc<bb8::Pool<RedisConnectionManager>>;

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

    fn headers(self, headers: HeaderMap) -> Self {
        Self {
            headers: Some(headers),
            ..self
        }
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub struct RateLimits {
    pub user_limit: Option<u32>,
    pub user_remaining: Option<u32>,
    pub user_reset: Option<u32>,
}

impl From<PocketyRateLimits> for RateLimits {
    fn from(limits: PocketyRateLimits) -> Self {
        Self {
            user_limit: limits.user_limit,
            user_remaining: limits.user_remaining,
            user_reset: limits.user_reset,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub struct WithRateLimits<T> {
    pub rate_limits: RateLimits,
    pub data: T,
}

#[derive(Clone)]
pub struct Config {
    pub jws_signing_secret: Secret,
    pub jwe_encryption_key: JWK<OAuthState>,
}

#[derive(Clone)]
pub struct AppState {
    pub pockety: Pockety,
    pub session_store: Cache,
    pub db: Store,
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

impl FromRef<AppState> for Cache {
    fn from_ref(state: &AppState) -> Self {
        state.session_store.clone()
    }
}

impl FromRef<AppState> for Store {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

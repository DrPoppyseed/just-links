use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    headers::{self},
    http::{request::Parts, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
    RequestPartsExt,
    TypedHeader,
};
use biscuit::{jwk::JWK, Empty};
use error::Error;
use pockety::Pockety;
use serde::{Deserialize, Serialize};

pub mod api;
pub mod error;

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
    pub jwe_encryption_key: JWK<Empty>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionData {
    request_token: Option<String>,
    access_token: Option<String>,
    csrf_token: Option<String>,
    username: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionData
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        tracing::debug!("start request");

        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match e.reason() {
                TypedHeaderRejectionReason::Missing => {
                    tracing::debug!("missing Cookie header");
                    Error::Cookie("missing Cookie header".to_string())
                }
                _ => {
                    tracing::error!("unexpected error getting Cookie header(s): {e}");
                    Error::Cookie("unexpected error getting Cookie header(s): {e}".to_string())
                }
            })?;

        tracing::debug!("found cookies: {cookies:#?}");

        let session_cookie = cookies
            .get(COOKIE_NAME)
            .ok_or(Error::Cookie("missing cookie".to_string()))?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .ok()
            .flatten()
            .ok_or(Error::Cookie("failed to load session".to_string()))?;

        session
            .get::<SessionData>("session")
            .ok_or(Error::Cookie("session not found".to_string()))
    }
}

use async_session::{async_trait, MemoryStore, SessionStore};
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    headers,
    http::request::Parts,
    RequestPartsExt,
    TypedHeader,
};
use error::Error;
use serde::{Deserialize, Serialize};

use crate::{error, COOKIE_NAME};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionData {
    pub request_token: Option<String>,
    pub access_token: Option<String>,
    pub csrf_token: Option<String>,
    pub username: Option<String>,
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

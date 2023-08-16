use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    headers,
    http::request::Parts,
    RequestPartsExt,
    TypedHeader,
};
use base64::{engine::general_purpose, Engine as _};
use base64ct::{Base64, Encoding};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use error::Error;
use futures::TryFutureExt;
use rand::{thread_rng, RngCore};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::{error, COOKIE_NAME};

pub type ConnPool = Pool<RedisConnectionManager>;

const SESSION_ID_LEN: usize = 64;

#[derive(Debug, Clone)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Default)]
pub struct HashedSessionId(pub String);

pub fn generate_session_id() -> Result<(SessionId, HashedSessionId), Error> {
    let mut session_id = vec![0u8; SESSION_ID_LEN];
    thread_rng().fill_bytes(&mut session_id);
    let session_id = general_purpose::URL_SAFE.encode(session_id);
    let hashed = hash(&session_id);

    Ok((SessionId(session_id), HashedSessionId(hashed)))
}

pub fn hash(input: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(input);
    Base64::encode_string(&hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RequestTokenSessionData {
    pub request_token: String,
    pub csrf_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthzedSessionData {
    pub access_token: String,
    pub username: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthzedSessionData
where
    Arc<ConnPool>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .map_err(|e| match e.reason() {
                TypedHeaderRejectionReason::Missing => {
                    tracing::debug!("missing Cookie header");
                    Error::Cookie("missing Cookie header".to_string())
                }
                _ => {
                    tracing::error!("unexpected error getting Cookie header(s): {e}");
                    Error::Cookie("unexpected error getting Cookie header(s): {e}".to_string())
                }
            })
            .await?;

        tracing::debug!("found cookies: {cookies:#?}");

        let session_cookie = cookies
            .get(COOKIE_NAME)
            .ok_or(Error::Cookie("missing cookie".to_string()))?;

        let pool = Arc::<ConnPool>::from_ref(state);
        let mut con = pool
            .get_owned()
            .map_err(|e| {
                tracing::error!(
                    "Failed to establish connection from connection pool. Error: {e:?}"
                );
                Error::Session("Failed to establish connection from connection pool".to_string())
            })
            .await?;

        let hashed_session_id = hash(session_cookie);
        let session_data: AuthzedSessionData = con
            .get(hashed_session_id)
            .map_err(|e| {
                tracing::error!("Failed to get SessionData with key: {session_cookie}. Error: {e}");
                Error::Session("Failed to get SessionData".to_string())
            })
            .and_then(|v: String| async move {
                serde_json::from_str(v.as_str()).map_err(|e| {
                    tracing::error!("Failed to deserialize string into SessionData. Error: {e}");
                    Error::Session("Failed to deserialize. internal error!".to_string())
                })
            })
            .await?;

        Ok(session_data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash() {
        let input = "input".to_string();
        let output = hash(&input);
        assert_ne!(input, output);
    }

    #[test]
    fn test_generate_session_id() {
        let (session_id, hashed_session_id) = generate_session_id().unwrap();
        assert_ne!(session_id.0, hashed_session_id.0);
        assert_eq!(hash(&session_id.0), hashed_session_id.0);
    }
}

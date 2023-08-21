use std::sync::Arc;

use axum::{
    extract::State,
    headers,
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    Json, TypedHeader,
};
use axum_extra::extract::cookie::{Cookie, Expiration};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::TryFutureExt;
use once_cell::sync::Lazy;
use pockety::{
    GetAccessTokenResponse as PocketyGetAccessTokenResponse,
    GetRequestTokenResponse as PocketyGetRequestTokenResponse, Pockety,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tracing::{debug, error, info};

use crate::{
    error::{ApiError, Error},
    oauth::{generate_csrf_token, OAuthState},
    session::{generate_session_id, hash, AuthzedSessionData, ConPool, RequestTokenSessionData},
    ApiResult, Config, TypedResponse, SESSION_ID_COOKIE_NAME,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRequestTokenResponse {
    pub request_token: String,
    pub auth_uri: String,
}

pub async fn get_request_token(
    State(pockety): State<Pockety>,
    State(config): State<Config>,
    State(session_store): State<Arc<ConPool>>,
) -> ApiResult<()> {
    const LOG_TAG: &str = "[get_request_token]";

    // TODO: implement rate limiting
    // TODO: is there a way to check if a user is already authed?
    // TODO: use an actual cookie manager, since we're currently not signing or encrypting them
    let PocketyGetRequestTokenResponse { code, .. } = pockety
        .get_request_token(None)
        .inspect_ok(|r| debug!("{LOG_TAG} got request token from pocket: res: {r:?}"))
        .inspect_err(|e| debug!("{LOG_TAG} failed to get request token from pocket. err: {e}"))
        .await?;

    let csrf_token = generate_csrf_token();

    let session_data = RequestTokenSessionData {
        request_token: code.clone(),
        csrf_token: csrf_token.clone(),
    };
    let (session_id, hashed_session_id) = generate_session_id()?;

    let mut con = session_store
        .get_owned()
        .map_err(|e| {
            error!("{LOG_TAG} Failed to established redis connection from pool. Error: {e:?}");
            Error::Session("Connection error".to_string())
        })
        .await?;

    let stringified_session_data = serde_json::to_string(&session_data)?;
    con.set(hashed_session_id.0.clone(), stringified_session_data)
        .inspect_ok(|_| {
            info!(
                "{LOG_TAG} Set new session: {} to store!",
                hashed_session_id.0
            );
        })
        .map_err(|e| {
            error!("{LOG_TAG} Failed to store session. Error: {e:?}");
            Error::Session("Failed to store session".to_string())
        })
        .await?;

    debug!("{LOG_TAG} generated following session_data: {session_data:?}");

    // state
    let token = OAuthState::new(code.clone(), session_id.0, csrf_token)
        .into_token(config.jws_signing_secret, config.jwe_encryption_key)?;

    debug!("{LOG_TAG} create encrypted token: {token:?}");

    let auth_uri: HeaderValue = format!(
        "{}?request_token={code}&redirect_uri={}%3Fstate={token}&state={token}",
        Pockety::AUTHORIZE_URL,
        pockety.redirect_url,
    )
    .parse()
    .map_err(|e| {
        Error::Api(ApiError::InternalServerError(format!(
            "Failed parse header value: {e}"
        )))
    })?;

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, auth_uri);

    Ok(TypedResponse::new(None)
        .headers(Some(headers))
        .status_code(StatusCode::SEE_OTHER))
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenRequest {
    pub state: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenResponse {
    pub username: String,
}

pub async fn get_access_token(
    State(pockety): State<Pockety>,
    State(config): State<Config>,
    State(session_store): State<Arc<Pool<RedisConnectionManager>>>,
    body: Json<GetAccessTokenRequest>,
) -> ApiResult<GetAccessTokenResponse> {
    const LOG_TAG: &str = "[get_access_token]";

    let OAuthState {
        request_token,
        session_id,
        csrf_token,
    } = OAuthState::from_token(
        body.state.clone(),
        config.jws_signing_secret,
        config.jwe_encryption_key,
    )?;

    // Make sure the session is valid before requesting the access token from pocket.com
    let mut con = session_store
        .get_owned()
        .map_err(|e| {
            error!("{LOG_TAG} Failed to established redis connection from pool. Error: {e:?}");
            Error::Session("Connection error".to_string())
        })
        .await?;

    let hashed_session_id = hash(&session_id);
    let hashed_session_id = hashed_session_id.as_str();
    let session_data = con
        .get(hashed_session_id)
        .inspect_ok(|_| {
            info!("{LOG_TAG} Found session: {hashed_session_id} in store!")
        })
        .map_err(|e| {
           error!("{LOG_TAG} failed to retrieve session: {hashed_session_id} with error: {e:?}");
            Error::Session("Failed to get session.".to_string())
        })
        .and_then(|session_data: String| async move {
            serde_json::from_str::<RequestTokenSessionData>(&session_data).map_err(|e| {
                error!("{LOG_TAG} failed to deserialize session: {hashed_session_id} with error: {e:?}");
                Error::Session("Failed to deserialize session.".to_string())
            })
        })
        .await?;

    // compare csrf_token in request with csrf_token in session
    if csrf_token != session_data.csrf_token.clone() {
        return Err(Error::Api(ApiError::Unauthorized(
            "CSRF token doesn't match".to_string(),
        )));
    }

    let res: PocketyGetAccessTokenResponse = pockety
        .get_access_token(request_token.clone())
        .inspect_ok(|r| debug!("{LOG_TAG} successfully acquired access_token from pocket: {r:?}"))
        .inspect_err(|e| debug!("{LOG_TAG} failed to get access_token from pocket: {e:?}"))
        .map_err(Error::from)
        .await?;

    // create new session with new crsf token and destroy previous session
    con.del(hashed_session_id)
        .map_err(|e| {
            // TODO: consider whether failing to destroy an old session should fail the entire
            // request
            error!("{LOG_TAG} failed to destroy session: {hashed_session_id} with error: {e:?}");
            Error::Session("Failed to destroy session.".to_string())
        })
        .await?;

    let (session_id, hashed_session_id) = generate_session_id()?;
    let session_data = AuthzedSessionData {
        access_token: res.access_token.clone(),
        username: res.username.clone(),
    };

    let stringified_session_data = serde_json::to_string(&session_data)?;
    con.set(hashed_session_id.0.clone(), stringified_session_data)
        .map_err(|e| {
            error!(
                "{LOG_TAG} failed to set new session: {} with error: {e:?}",
                hashed_session_id.0
            );
            Error::Session("Failed to set new session.".to_string())
        })
        .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        Cookie::build(SESSION_ID_COOKIE_NAME, session_id.0)
            .http_only(true)
            .expires(Expiration::from(
                OffsetDateTime::now_utc() + Duration::minutes(60),
            ))
            .path("/")
            .finish()
            .to_string()
            .parse()
            .unwrap(),
    );

    Ok(TypedResponse::new(Some(GetAccessTokenResponse {
        username: res.username.clone(),
    }))
    .headers(Some(headers)))
}

#[derive(Serialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetSessionResponse {
    has_session: bool,
    username: Option<String>,
}

static NOT_AUTHZED_RESPONSE: Lazy<TypedResponse<GetSessionResponse>> =
    Lazy::new(|| TypedResponse::new(Some(GetSessionResponse::default())));

// responds with user authenticated or not
pub async fn get_session(
    State(session_store): State<Arc<Pool<RedisConnectionManager>>>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> ApiResult<GetSessionResponse> {
    const LOG_TAG: &str = "[get_session]";

    let session_cookie = match cookies.get(SESSION_ID_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return Ok(NOT_AUTHZED_RESPONSE.clone()),
    };

    let mut con = match session_store.get_owned().await {
        Ok(con) => con,
        Err(_) => return Ok(NOT_AUTHZED_RESPONSE.clone()),
    };

    let hashed_session_id = hash(session_cookie);
    match con
        .get(hashed_session_id)
        .map_err(|e| {
            tracing::error!(
                "{LOG_TAG} Failed to get SessionData with key: {session_cookie}. Error: {e}"
            );
            Error::Session("Failed to get SessionData".to_string())
        })
        .and_then(|v: String| async move {
            serde_json::from_str::<AuthzedSessionData>(v.as_str()).map_err(|e| {
                tracing::error!(
                    "{LOG_TAG} Failed to deserialize string into SessionData. Error: {e}"
                );
                Error::Session("Failed to deserialize. internal error!".to_string())
            })
        })
        .await
    {
        Ok(session_data) => Ok(TypedResponse::new(Some(GetSessionResponse {
            has_session: true,
            username: Some(session_data.username),
        }))),
        Err(_) => Ok(NOT_AUTHZED_RESPONSE.clone()),
    }
}

use std::sync::Arc;

use axum::{
    extract::State,
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap,
        HeaderValue,
        StatusCode,
    },
    Json,
};
use axum_extra::extract::cookie::{Cookie, Expiration};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::TryFutureExt;
use pockety::{
    GetAccessTokenResponse as PocketyGetAccessTokenResponse,
    GetRequestTokenResponse as PocketyGetRequestTokenResponse,
    Pockety,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tracing::{debug, error};

use crate::{
    error::{ApiError, Error},
    oauth::{generate_csrf_token, OAuthState},
    session::{generate_session_id, hash, AuthzedSessionData, RequestTokenSessionData},
    ApiResult,
    AppState,
    Config,
    TypedResponse,
    COOKIE_NAME,
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
    State(session_store): State<Arc<Pool<RedisConnectionManager>>>,
) -> ApiResult<()> {
    const LOG_TAG: &str = "[get_request_token]";
    debug!("{LOG_TAG} start!");

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
    con.set(hashed_session_id.0, stringified_session_data)
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
    pub state: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenResponse {
    pub username: String,
    pub state: Option<String>,
}

#[axum::debug_handler(state = AppState)]
pub async fn get_access_token(
    State(pockety): State<Pockety>,
    State(config): State<Config>,
    State(session_store): State<Arc<Pool<RedisConnectionManager>>>,
    body: Json<GetAccessTokenRequest>,
) -> ApiResult<GetAccessTokenResponse> {
    const LOG_TAG: &str = "[get_access_token]";

    debug!("{LOG_TAG} start! received request body: {body:?}");

    let OAuthState {
        request_token,
        session_id,
        csrf_token,
    } = body
        .state
        .clone()
        .ok_or(Error::Api(ApiError::BadRequest(
            "Missing state param".to_string(),
        )))
        .and_then(|token| {
            OAuthState::from_token(token, config.jws_signing_secret, config.jwe_encryption_key)
        })?;

    // Make sure the session is valid before requesting the access token from pocket.com
    let mut con = session_store
        .get_owned()
        .map_err(|e| {
            error!("{LOG_TAG} Failed to established redis connection from pool. Error: {e:?}");
            Error::Session("Connection error".to_string())
        })
        .await?;

    let hashed_session_id = hash(&session_id)?;
    let hashed_session_id = hashed_session_id.as_str();
    let session_data = con
        .get(hashed_session_id.clone())
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
    // TODO: probably use matches!
    if csrf_token != session_data.csrf_token.clone() {
        return Err(Error::Api(ApiError::Unauthorized(
            "CSRF token doesn't match".to_string(),
        )));
    }

    // TODO: Should I fill `state` in?
    let res: PocketyGetAccessTokenResponse = pockety
        .get_access_token(request_token.clone(), None)
        .inspect_ok(|r| debug!("{LOG_TAG} successfully acquired access_token from pocket: {r:?}"))
        .inspect_err(|e| debug!("{LOG_TAG} failed to get access_token from pocket: {e:?}"))
        .map_err(Error::from)
        .await?;

    // TODO: create new session with new crsf token, and destroy previous session

    let _destroy = con
        .del(hashed_session_id.clone())
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
        username: Some(res.username.clone()),
    };

    let stringified_session_data = serde_json::to_string(&session_data)?;
    let _set = con
        .set(hashed_session_id.0.clone(), stringified_session_data)
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
        Cookie::build(COOKIE_NAME, session_id.0)
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
        state: res.state,
    }))
    .headers(Some(headers)))
}

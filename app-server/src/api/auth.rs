use async_session::{MemoryStore, Session, SessionStore};
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
use futures::TryFutureExt;
use pockety::{
    GetAccessTokenResponse as PocketyGetAccessTokenResponse,
    GetRequestTokenResponse as PocketyGetRequestTokenResponse,
    Pockety,
};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tracing::debug;

use crate::{
    error::{ApiError, Error},
    oauth::{generate_csrf_token, OAuthState},
    session::SessionData,
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
    State(store): State<MemoryStore>,
    State(pockety): State<Pockety>,
    State(config): State<Config>,
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
    let mut session: Session = Session::new();
    let session_data = SessionData {
        request_token: Some(code.clone()),
        csrf_token: Some(csrf_token.clone()),
        ..Default::default()
    };

    debug!("{LOG_TAG} generated following session_data: {session_data:?}");

    // session
    // TODO: abstract this probably, since we don't want to handle CRUD with session
    // through raw string keys every time right?
    session.insert("session", &session_data)?;
    let session_cookie = store
        .store_session(session)
        .inspect_ok(|r| {
            debug!("{LOG_TAG} successfully stored session to store and generated cookie: {r:?}")
        })
        .inspect_err(|e| debug!("{LOG_TAG} failed to store session with error: {e:?}"))
        .await
        .ok()
        .flatten()
        .ok_or(Error::Api(ApiError::InternalServerError(
            "Failed to store session".to_string(),
        )))?;

    // state
    let token = OAuthState::new(code.clone(), session_cookie, csrf_token)
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
    State(store): State<MemoryStore>,
    State(config): State<Config>,
    body: Json<GetAccessTokenRequest>,
) -> ApiResult<GetAccessTokenResponse> {
    const LOG_TAG: &str = "[get_access_token]";

    debug!("{LOG_TAG} start! received request body: {body:?}");

    let OAuthState {
        request_token,
        session_cookie,
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

    // TODO: Should I fill `state` in?
    let res: PocketyGetAccessTokenResponse = pockety
        .get_access_token(request_token.clone(), None)
        .inspect_ok(|r| debug!("{LOG_TAG} successfully acquired access_token from pocket: {r:?}"))
        .inspect_err(|e| debug!("{LOG_TAG} failed to get access_token from pocket: {e:?}"))
        .map_err(Error::from)
        .await?;

    let mut session = store
        .clone()
        .load_session(session_cookie.clone())
        .inspect_err(|e| debug!("failed to store session with error: {e:?}"))
        .await
        .ok()
        .flatten()
        .ok_or(Error::Api(ApiError::InternalServerError(
            "Couldn't find the session".to_string(),
        )))?;

    // retrieve session by cookie value
    let mut session_data: SessionData =
        session
            .get("session")
            .ok_or(Error::Api(ApiError::InternalServerError(
                "Empty session".to_string(),
            )))?;

    // compare csrf_token in request with csrf_token in session
    // TODO: probably use matches!
    if csrf_token
        != session_data
            .csrf_token
            .clone()
            .ok_or(Error::Api(ApiError::InternalServerError(
                "CSRF token missing in session".to_string(),
            )))?
    {
        return Err(Error::Api(ApiError::Unauthorized(
            "CSRF token doesn't match".to_string(),
        )));
    }

    // update session
    session_data.access_token = Some(res.access_token.clone());
    session.insert("session", &session_data)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        Cookie::build(COOKIE_NAME, session_cookie)
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

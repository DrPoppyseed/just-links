use async_session::{
    chrono::{Duration, Utc},
    MemoryStore,
    Session,
    SessionStore,
};
use axum::{
    extract::State,
    headers,
    http::{
        header::{LOCATION, SET_COOKIE},
        HeaderMap,
        StatusCode,
    },
    response::IntoResponse,
    TypedHeader,
};
use futures::TryFutureExt;
use pockety::{
    models::PocketItem,
    GetRequestTokenResponse as PocketyGetRequestTokenResponse,
    Pockety,
    PocketyUrl,
};
use serde::Serialize;

use crate::{
    error::{self, Error},
    ApiResult,
    SessionData,
    TypedResponse,
    COOKIE_NAME,
};

pub async fn health_check() -> impl IntoResponse {
    "Healthy!"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRequestTokenResponse {
    request_token: String,
    auth_uri: String,
}

pub async fn get_request_token(
    State(store): State<MemoryStore>,
    State(pockety): State<Pockety>,
) -> ApiResult<()> {
    // TODO: implement rate limiting
    // TODO: is there a way to check if a user is already authed?
    // TODO: use an actual cookie manager, since we're currently not signing or encrypting them
    let PocketyGetRequestTokenResponse { code, .. } = pockety.get_request_token(None).await?;

    let auth_uri = format!(
        "{}?request_token={}&redirect_uri={}",
        PocketyUrl::AUTHORIZE,
        code,
        pockety.redirect_url
    );

    let session_data = SessionData {
        request_token: Some(code.clone()),
        ..Default::default()
    };
    let mut session = Session::new();
    session.insert("session", &session_data)?;

    let cookie_expiration = Utc::now() + Duration::hours(1);
    let cookie = store
        .store_session(session)
        .await
        .ok()
        .flatten()
        .ok_or(Error::Cookie("failed to store session".to_string()))?;
    let cookie = format!(
        "{COOKIE_NAME}={cookie}; Expires={cookie_expiration}; SameSite=Lax; HttpOnly; Secure"
    );

    tracing::debug!("session created with cookie: {cookie} and request_token: {code}");

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers.insert(LOCATION, auth_uri.parse().unwrap());

    Ok(TypedResponse::new(None)
        .headers(Some(headers))
        .status_code(StatusCode::SEE_OTHER))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenResponse {
    username: String,
    state: Option<String>,
}

pub async fn get_access_token(
    State(pockety): State<Pockety>,
    State(store): State<MemoryStore>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    session_data: SessionData,
) -> ApiResult<GetAccessTokenResponse> {
    if let Some(request_token) = session_data.request_token.clone() {
        // TODO: fill in the state param
        pockety
            .get_access_token(request_token.clone(), None)
            .and_then(|res| async move {
                // destroy old session
                let cookie = cookies.get(COOKIE_NAME).unwrap();
                let session = store
                    .load_session(cookie.to_string())
                    .await
                    .unwrap()
                    .unwrap();
                store.destroy_session(session).await.unwrap();

                // create new session
                let mut session = Session::new();
                session
                    .insert(
                        "session",
                        &SessionData {
                            request_token: Some(request_token),
                            access_token: Some(res.access_token.clone()),
                            username: Some(res.username.clone()),
                        },
                    )
                    .unwrap();

                let cookie = store
                    .store_session(session)
                    .await
                    .ok()
                    .flatten()
                    .ok_or(Error::Cookie("failed to store session".to_string()))
                    .unwrap();
                let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; HttpOnly; Secure");

                Ok((res, cookie))
            })
            .map_ok(|(res, cookie)| {
                let mut headers = HeaderMap::new();
                headers.insert(SET_COOKIE, cookie.parse().unwrap());

                TypedResponse::new(Some(GetAccessTokenResponse {
                    username: res.username,
                    state: res.state,
                }))
                .headers(Some(headers))
            })
            .map_err(Error::from)
            .await
    } else {
        Err(error::Error::Pockety(format!(
            "I couldn't find your request token"
        )))
    }
}

#[derive(Serialize)]
pub struct GetArticlesResponse {
    articles: Vec<PocketItem>,
}

pub async fn get_articles(
    State(pockety): State<Pockety>,
    session_data: SessionData,
) -> ApiResult<GetArticlesResponse> {
    if let Some(access_token) = session_data.access_token {
        let since = Utc::now() - Duration::days(7);

        pockety
            .retrieve()
            .access_token(access_token)
            .since(since)
            .execute()
            .map_ok(|articles| TypedResponse::new(Some(GetArticlesResponse { articles })))
            .map_err(Error::from)
            .await
    } else {
        Err(error::Error::Pockety(format!(
            "I couldn't find your access_token"
        )))
    }
}

#[derive(Serialize)]
pub struct GetSessionInfoResponse {
    pub username: Option<String>,
}

pub async fn get_session_info(session_data: SessionData) -> TypedResponse<GetSessionInfoResponse> {
    TypedResponse::new(Some(GetSessionInfoResponse {
        username: session_data.username,
    }))
}

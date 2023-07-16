use async_session::{
    chrono::{Duration, Utc},
    MemoryStore,
    Session,
    SessionStore,
};
use axum::{
    extract::{self, State},
    http::{header::SET_COOKIE, HeaderMap},
    response::IntoResponse,
};
use pockety::{models::PocketItem, Pockety, PocketyUrl};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{error::Error, ApiResult, SessionData, TypedResponse, COOKIE_NAME};

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
    State(pockety): State<Pockety>,
) -> ApiResult<GetRequestTokenResponse> {
    let request_token = pockety.get_request_token(None).await?;

    let auth_uri = format!(
        "{}?request_token={request_token}&redirect_uri={}",
        PocketyUrl::AUTHORIZE,
        pockety.redirect_url
    );

    let response = GetRequestTokenResponse {
        request_token,
        auth_uri,
    };

    Ok(TypedResponse {
        body: Some(response),
        ..Default::default()
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenResponse {
    access_token: String,
    session_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenRequest {
    request_token: String,
}

pub async fn get_access_token(
    State(store): State<MemoryStore>,
    State(pockety): State<Pockety>,
    extract::Json(request): extract::Json<GetAccessTokenRequest>,
) -> ApiResult<GetAccessTokenResponse> {
    let access_token = pockety
        .get_access_token(&request.request_token, None)
        .await?;

    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let session_data = SessionData {
        session_id,
        access_token: access_token.clone(),
    };

    let mut session = Session::new();
    session.insert("session", &session_data)?;

    let cookie = store
        .store_session(session)
        .await
        .ok()
        .flatten()
        .ok_or(Error::Cookie("Failed to store session".to_string()))?;
    let cookie = format!("{COOKIE_NAME}={cookie}; SameSite=Lax; Path=/; HttpOnly");

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    let response = GetAccessTokenResponse {
        access_token,
        session_id: session_data.session_id,
    };

    Ok(TypedResponse {
        body: Some(response),
        ..Default::default()
    })
}

#[derive(Serialize)]
pub struct GetArticlesResponse {
    articles: Vec<PocketItem>,
}

pub async fn get_articles(State(pockety): State<Pockety>) -> ApiResult<GetArticlesResponse> {
    let since = Utc::now() - Duration::days(7);
    let articles = pockety.retrieve().since(since).execute().await?;

    let response = GetArticlesResponse { articles };

    Ok(TypedResponse {
        body: Some(response),
        ..Default::default()
    })
}

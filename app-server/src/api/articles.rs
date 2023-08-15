use async_session::chrono::{Duration, Utc};
use axum::extract::State;
use futures::TryFutureExt;
use pockety::{models::PocketItem, Pockety};
use serde::Serialize;
use tracing::debug;

use crate::{error::Error, session::AuthzedSessionData, ApiResult, TypedResponse};

#[derive(Serialize)]
pub struct GetArticlesResponse {
    articles: Vec<PocketItem>,
}

pub async fn get_articles(
    State(pockety): State<Pockety>,
    session_data: AuthzedSessionData,
) -> ApiResult<GetArticlesResponse> {
    const LOG_TAG: &str = "[get_articles]";
    debug!("{LOG_TAG} start!");

    let since = Utc::now() - Duration::days(7);

    pockety
        .retrieve()
        .access_token(session_data.access_token)
        .since(since)
        .execute()
        .inspect_ok(|articles| debug!("{LOG_TAG} successfully fetched articles: {articles:?}"))
        .inspect_err(|e| debug!("{LOG_TAG} failed to fetch articles with error: {e:?}"))
        .map_ok(|articles| TypedResponse::new(Some(GetArticlesResponse { articles })))
        .map_err(Error::from)
        .await
}

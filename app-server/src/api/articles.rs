use async_session::chrono::{Duration, Utc};
use axum::extract::State;
use futures::TryFutureExt;
use pockety::{models::PocketItem, Pockety};
use serde::Serialize;

use crate::{error::Error, session::SessionData, ApiResult, TypedResponse};

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
        Err(Error::Pocket(format!("I couldn't find your access_token")))
    }
}

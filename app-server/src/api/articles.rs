use axum::extract::State;
use chrono::{Duration, Utc};
use futures::TryFutureExt;
use pockety::{
    models::{ItemHas, ItemImage, ItemStatus, ItemVideo, PocketItem, Timestamp},
    Pockety,
};
use serde::Serialize;
use tracing::debug;

use crate::{error::Error, session::AuthzedSessionData, ApiResult, TypedResponse};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub item_id: String,
    pub resolved_id: Option<String>,
    pub given_url: Option<String>,
    pub given_title: Option<String>,
    pub favorite: Option<String>,
    pub status: ItemStatus,
    pub time_added: Option<Timestamp>,
    pub time_updated: Option<Timestamp>,
    pub time_read: Option<Timestamp>,
    pub time_favorited: Option<Timestamp>,
    pub sort_id: Option<u32>,
    pub resolved_url: Option<String>,
    pub resolved_title: Option<String>,
    pub excerpt: Option<String>,
    pub is_article: Option<String>,
    pub is_index: Option<String>,
    pub has_image: Option<ItemHas>,
    pub has_video: Option<ItemHas>,
    pub word_count: Option<String>,
    pub tags: Option<String>,
    pub authors: Option<String>,
    pub images: Option<Vec<ItemImage>>,
    pub videos: Option<Vec<ItemVideo>>,
    pub lang: Option<String>,
    pub time_to_read: Option<u32>,
    pub listen_duration_estimate: Option<u32>,
    pub top_image_url: Option<String>,
    pub domain_metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct GetArticlesResponse {
    articles: Vec<PocketItem>,
}

pub async fn get_articles(
    State(pockety): State<Pockety>,
    session_data: AuthzedSessionData,
) -> ApiResult<GetArticlesResponse> {
    const LOG_TAG: &str = "[get_articles]";

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

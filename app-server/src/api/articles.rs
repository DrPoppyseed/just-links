use axum::extract::{Query, State};
use futures::TryFutureExt;
use pockety::{
    models::{ItemAuthor, ItemHas, ItemImage, ItemStatus, ItemVideo, PocketItem, Timestamp},
    Pockety,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{error::Error, session::AuthzedSessionData, ApiResult, TypedResponse};

#[derive(Serialize, Debug, Clone)]
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
    pub authors: Option<Vec<ItemAuthor>>,
    pub images: Option<Vec<ItemImage>>,
    pub videos: Option<Vec<ItemVideo>>,
    pub lang: Option<String>,
    pub time_to_read: Option<u32>,
    pub listen_duration_estimate: Option<u32>,
    pub top_image_url: Option<String>,
}

impl From<PocketItem> for Article {
    fn from(item: PocketItem) -> Self {
        Self {
            item_id: item.item_id.0,
            resolved_id: item.resolved_id.map(|id| id.0),
            given_url: item.given_url,
            given_title: item.given_title,
            favorite: item.favorite,
            status: item.status,
            time_added: item.time_added,
            time_updated: item.time_updated,
            time_read: item.time_read,
            time_favorited: item.time_favorited,
            sort_id: item.sort_id,
            resolved_url: item.resolved_url,
            resolved_title: item.resolved_title,
            excerpt: item.excerpt,
            is_article: item.is_article,
            is_index: item.is_index,
            has_image: item.has_image,
            has_video: item.has_video,
            word_count: item.word_count,
            tags: item.tags,
            authors: item.authors,
            images: item.images,
            videos: item.videos,
            lang: item.lang,
            time_to_read: item.time_to_read,
            listen_duration_estimate: item.listen_duration_estimate,
            top_image_url: item.top_image_url,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub page: u32,
}

impl Pagination {
    const PER_PAGE: u32 = 30;
}

#[derive(Serialize)]
pub struct GetArticlesResponse {
    articles: Vec<Article>,
}

pub async fn get_articles(
    State(pockety): State<Pockety>,
    pagination: Query<Pagination>,
    session_data: AuthzedSessionData,
) -> ApiResult<GetArticlesResponse> {
    const LOG_TAG: &str = "[get_articles]";

    let pagination: Pagination = pagination.0;

    pockety
        .retrieve()
        .access_token(session_data.access_token)
        .count(Pagination::PER_PAGE)
        .offset(Pagination::PER_PAGE * pagination.page)
        .execute()
        .inspect_err(|e| debug!("{LOG_TAG} failed to fetch articles with error: {e:?}"))
        .map_ok(|articles| {
            let articles = articles
                .into_iter()
                .map(Article::from)
                .filter(|article| article.given_title.is_some() && article.given_url.is_some())
                .collect();
            TypedResponse::new(Some(GetArticlesResponse { articles }))
        })
        .map_err(Error::from)
        .await
}

use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
};
use futures::{stream, Stream, StreamExt, TryFutureExt, TryStreamExt};
use pockety::{
    models::{ItemAuthor, ItemHas, ItemImage, ItemStatus, ItemVideo, PocketItem, Timestamp},
    Pockety,
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, time::Duration};
// use tokio_stream::StreamExt as _;
use tracing::{debug, error, info};

use crate::{
    db::{
        convert_article_to_article_author_models, convert_article_to_article_image_models,
        convert_article_to_article_model, convert_article_to_article_video_models, fetch_user,
        ArticleStore,
    },
    error::Error,
    session::AuthzedSessionData,
    ApiResult, Store, TypedResponse, WithRateLimits,
};

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
) -> ApiResult<WithRateLimits<GetArticlesResponse>> {
    const LOG_TAG: &str = "[get_articles]";

    let pagination: Pagination = pagination.0;

    pockety
        .retrieve()
        .access_token(session_data.access_token)
        .count(Pagination::PER_PAGE)
        .offset(Pagination::PER_PAGE * pagination.page)
        .execute()
        .map_ok(|res| {
            let rate_limits = res.rate_limits.into();
            let articles = res
                .data
                .into_iter()
                .map(Article::from)
                .filter(|article| article.given_title.is_some() && article.given_url.is_some())
                .collect::<Vec<_>>();

            info!(
                "{LOG_TAG} fetched {count} articles for user {username}. Current rate limits: {rate_limits:?}",
                LOG_TAG = LOG_TAG,
                count = articles.len(),
                username = session_data.username
            );

            TypedResponse::new(Some(WithRateLimits {
                data: GetArticlesResponse { articles },
                rate_limits
            }))
        })
        .inspect_err(|e| debug!("{LOG_TAG} failed to fetch articles with error: {e:?}"))
        .map_err(Error::from)
        .await
}

pub async fn simulate_sync_articles(
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    let ceil = 10;
    let stream = stream::iter(0..ceil)
        .enumerate()
        .then(move |(idx, _)| async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ok(Event::default().data(format!("{},{}", idx, ceil)))
        })
        .inspect_ok(|event| info!("Published event: {event:?}"))
        .inspect_err(|e| error!("Failed to publish event: {e:?}"));

    return Ok(Sse::new(stream).keep_alive(KeepAlive::default()));
}

pub async fn sync_articles(
    State(pockety): State<Pockety>,
    State(store): State<Store>,
    session_data: AuthzedSessionData,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Error> {
    const LOG_TAG: &str = "[sync_articles]";

    let user_id: i32 = match fetch_user(store.clone(), &session_data.username).await? {
        Some(user) => user.id,
        None => {
            return Err(Error::Db("User not found!".to_string()));
        }
    };

    let articles = pockety
        .retrieve()
        .access_token(session_data.access_token)
        .execute()
        .map_ok(|res| res.data)
        .inspect_err(|e| debug!("{LOG_TAG} Failed to fetch articles with error: {e:?}"))
        .await?;

    let article_len = articles.len();

    let stream = stream::iter(articles)
        .enumerate()
        .then(move |(idx, article)| {
            let store = store.clone();
            async move {
                let article = Article::from(article);
                let article_model =
                    convert_article_to_article_model(article.clone(), user_id).unwrap();
                let article_id = store.upsert_article(article_model).await.unwrap();

                let article_video_models =
                    convert_article_to_article_video_models(article.clone(), article_id).unwrap();
                for article_video_model in article_video_models {
                    let _ = store
                        .upsert_article_video(article_id, article_video_model)
                        .await;
                }

                let article_image_models =
                    convert_article_to_article_image_models(article.clone(), article_id).unwrap();
                for article_image_model in article_image_models {
                    let _ = store
                        .upsert_article_image(article_id, article_image_model)
                        .await;
                }

                let article_author_models =
                    convert_article_to_article_author_models(article, article_id).unwrap();
                for article_author_model in article_author_models {
                    let _ = store
                        .upsert_article_author(article_id, article_author_model)
                        .await;
                }

                Ok(Event::default().data(format!("{idx},{article_len}")))
            }
        });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
    // }
}

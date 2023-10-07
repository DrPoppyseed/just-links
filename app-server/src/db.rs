use futures::TryFutureExt;
use std::sync::Arc;

use sqlx::PgPool;
use tracing::{error, info};

use crate::{api::articles::Article, error::Error};

pub fn convert_article_to_article_model(
    article: Article,
    user_id: &str,
) -> Result<ArticleModel, Error> {
    let article_model = ArticleModel {
        user_id: user_id
            .parse()
            .map_err(|_| Error::Db("Failed to parse user_id to i32.".to_string()))?,
        item_id: article.item_id,
        resolved_id: article.resolved_id,
        given_url: article.given_url,
        given_title: article.given_title,
        favorite: article
            .favorite
            .and_then(|favorite| {
                favorite
                    .parse()
                    .map_err(|_| Error::Db("Failed to parse favorite to bool.".to_string()))
                    .ok()
            })
            .unwrap_or(false),
        status: article.status.as_u8() as i32,
        time_added: article.time_added.map(|time| time.0),
        time_updated: article.time_updated.map(|time| time.0),
        time_read: article.time_read.map(|time| time.0),
        time_favorited: article.time_favorited.map(|time| time.0),
        sort_id: article.sort_id.map(|sort_id| sort_id as i32),
        resolved_url: article.resolved_url,
        resolved_title: article.resolved_title,
        excerpt: article.excerpt,
        is_article: article
            .is_article
            .map(|is_article| match is_article.as_str() {
                "0" => false,
                _ => true,
            })
            .unwrap_or(true),
        is_index: article
            .is_index
            .map(|is_index| match is_index.as_str() {
                "0" => false,
                _ => true,
            })
            .unwrap_or(true),
        has_image: article.has_image.map(|has_image| has_image.as_u8() as i32),
        has_video: article.has_video.map(|has_video| has_video.as_u8() as i32),
        word_count: article
            .word_count
            .and_then(|word_count| word_count.parse().ok()),
        tags: article.tags,
        lang: article.lang,
        time_to_read: article.time_to_read.map(|time| time as i32),
        listen_duration_estimate: article.listen_duration_estimate.map(|time| time as i32),
        top_image_url: article.top_image_url,
    };
    Ok(article_model)
}

pub fn convert_article_to_article_video_models(
    article: Article,
    article_id: i32,
) -> Result<Vec<ArticleVideoModel>, Error> {
    let article_video_model: Vec<ArticleVideoModel> = article
        .videos
        .map(|videos| {
            videos
                .into_iter()
                .map(|video| ArticleVideoModel {
                    article_id,
                    item_id: video.item_id.0,
                    video_id: video.video_id.0,
                    src: video.src,
                    height: video
                        .height
                        .parse()
                        .map_err(|e| {
                            error!("Failed to parse height to i32. Error: {e:?}");
                        })
                        .unwrap_or(0),
                    width: video
                        .width
                        .parse()
                        .map_err(|e| {
                            error!("Failed to parse width to i32. Error: {e:?}");
                        })
                        .unwrap_or(0),
                    length: video.length.and_then(|length| length.parse().ok()),
                    vid: video.vid,
                })
                .collect()
        })
        .unwrap_or(Vec::new());

    Ok(article_video_model)
}

pub fn convert_article_to_article_image_models(
    articles: Article,
    article_id: i32,
) -> Result<Vec<ArticleImageModel>, Error> {
    let article_image_models: Vec<ArticleImageModel> = articles
        .images
        .map(|images| {
            images
                .into_iter()
                .map(|image| ArticleImageModel {
                    article_id,
                    item_id: image.item_id.0,
                    image_id: image.image_id.0,
                    src: image.src,
                    height: image
                        .height
                        .parse()
                        .map_err(|e| {
                            error!("Failed to parse height to i32. Error: {e:?}");
                        })
                        .unwrap_or(0),
                    width: image
                        .width
                        .parse()
                        .map_err(|e| {
                            error!("Failed to parse width to i32. Error: {e:?}");
                        })
                        .unwrap_or(0),
                    credit: image.credit,
                    caption: image.caption,
                })
                .collect()
        })
        .unwrap_or(Vec::new());

    Ok(article_image_models)
}

pub fn convert_article_to_article_author_models(
    articles: Article,
    article_id: i32,
) -> Result<Vec<ArticleAuthorModel>, Error> {
    let article_author_models: Vec<ArticleAuthorModel> = articles
        .authors
        .map(|authors| {
            authors
                .into_iter()
                .map(|author| ArticleAuthorModel {
                    article_id,
                    author_id: author.id.0,
                    name: author.name,
                    url: author.url,
                })
                .collect()
        })
        .unwrap_or(Vec::new());

    Ok(article_author_models)
}

#[derive(Debug, Clone)]
pub struct ArticleModel {
    pub user_id: i32,
    pub item_id: String,
    pub resolved_id: Option<String>,
    pub given_url: Option<String>,
    pub given_title: Option<String>,
    pub favorite: bool,
    pub status: i32,
    pub time_added: Option<i64>,
    pub time_updated: Option<i64>,
    pub time_read: Option<i64>,
    pub time_favorited: Option<i64>,
    pub sort_id: Option<i32>,
    pub resolved_url: Option<String>,
    pub resolved_title: Option<String>,
    pub excerpt: Option<String>,
    pub is_article: bool,
    pub is_index: bool,
    pub has_image: Option<i32>,
    pub has_video: Option<i32>,
    pub word_count: Option<i32>,
    pub tags: Option<String>,
    pub lang: Option<String>,
    pub time_to_read: Option<i32>,
    pub listen_duration_estimate: Option<i32>,
    pub top_image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArticleVideoModel {
    pub article_id: i32,
    pub item_id: String,
    pub video_id: String,
    pub src: String,
    pub height: i32,
    pub width: i32,
    pub length: Option<i32>,
    pub vid: String,
}

#[derive(Debug, Clone)]
pub struct ArticleImageModel {
    pub article_id: i32,
    pub item_id: String,
    pub image_id: String,
    pub src: String,
    pub height: i32,
    pub width: i32,
    pub credit: String,
    pub caption: String,
}

#[derive(Debug, Clone)]
pub struct ArticleAuthorModel {
    pub article_id: i32,
    pub author_id: String,
    pub name: String,
    pub url: String,
}

pub async fn create_new_user_if_not_exists(
    pool: Arc<PgPool>,
    username: &str,
) -> Result<i32, Error> {
    let res = sqlx::query!(
        r#"
    INSERT INTO users (username)
    VALUES ($1)
    ON CONFLICT DO NOTHING
    RETURNING id
            "#,
        username
    )
    .fetch_one(&*pool)
    .map_err(|e| {
        error!("Failed to create new user. Error: {e:?}");
        Error::Db("Failed to create new user.".to_string())
    })
    .inspect_ok(|res| info!("Created new user with username: {username}, id: {}", res.id))
    .await?;

    Ok(res.id)
    // Ok(0)
}

pub async fn batch_sync_articles(
    pool: Arc<PgPool>,
    user_id: &str,
    articles: Vec<Article>,
) -> Result<(), Error> {
    for article in articles {
        let article_model = convert_article_to_article_model(article.clone(), user_id)?;

        let user_id: i32 = user_id
            .parse()
            .map_err(|_| Error::Db("Failed to parse user_id to i32.".to_string()))?;

        let article_id = sqlx::query!(
            r#"
            INSERT INTO pocket_articles (
                user_id,
                item_id,
                resolved_id,
                given_url,
                given_title,
                favorite,
                status,
                time_added,
                time_updated,
                time_read,
                time_favorited,
                sort_id,
                resolved_url,
                resolved_title,
                excerpt,
                is_article,
                is_index,
                has_image,
                has_video,
                word_count,
                lang,
                time_to_read,
                listen_duration_estimate,
                top_image_url
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                $12,
                $13,
                $14,
                $15,
                $16,
                $17,
                $18,
                $19,
                $20,
                $21,
                $22,
                $23,
                $24
            )
            ON CONFLICT (
                user_id,
                item_id
            )
            DO UPDATE 
            SET
            resolved_id = EXCLUDED.resolved_id,
            given_url = EXCLUDED.given_url,
            given_title = EXCLUDED.given_title,
            favorite = EXCLUDED.favorite,
            status = EXCLUDED.status,
            time_added = EXCLUDED.time_added,
            time_updated = EXCLUDED.time_updated,
            time_read = EXCLUDED.time_read,
            time_favorited = EXCLUDED.time_favorited,
            sort_id = EXCLUDED.sort_id,
            resolved_url = EXCLUDED.resolved_url,
            resolved_title = EXCLUDED.resolved_title,
            excerpt = EXCLUDED.excerpt,
            is_article = EXCLUDED.is_article,
            is_index = EXCLUDED.is_index,
            has_image = EXCLUDED.has_image,
            has_video = EXCLUDED.has_video,
            word_count = EXCLUDED.word_count,
            lang = EXCLUDED.lang,
            time_to_read = EXCLUDED.time_to_read,
            listen_duration_estimate = EXCLUDED.listen_duration_estimate,
            top_image_url = EXCLUDED.top_image_url 
            RETURNING id"#,
            user_id,
            article_model.item_id,
            article_model.resolved_id,
            article_model.given_url,
            article_model.given_title,
            article_model.favorite,
            article_model.status,
            article_model.time_added,
            article_model.time_updated,
            article_model.time_read,
            article_model.time_favorited,
            article_model.sort_id,
            article_model.resolved_url,
            article_model.resolved_title,
            article_model.excerpt,
            article_model.is_article,
            article_model.is_index,
            article_model.has_image,
            article_model.has_video,
            article_model.word_count,
            article_model.lang,
            article_model.time_to_read,
            article_model.listen_duration_estimate,
            article_model.top_image_url
        )
        .fetch_one(&*pool)
        .map_err(|e| {
            error!("Failed to insert articles. Error: {e:?}");
            Error::Db("Failed to insert articles.".to_string())
        })
        .await?;

        let article_video_models =
            convert_article_to_article_video_models(article.clone(), article_id.id)?;

        for article_video_model in article_video_models {
            let _ = sqlx::query!(
                r#"
            INSERT INTO 
            pocket_article_videos (
                pocket_article_id,
                item_id,
                video_id,
                src,
                height,
                width,
                length,
                vid
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8
            )
            ON CONFLICT (
                pocket_article_id, 
                item_id, 
                video_id
            ) 
            DO UPDATE
            SET 
            src = EXCLUDED.src,
            height = EXCLUDED.height, 
            width = EXCLUDED.width, 
            length = EXCLUDED.length, 
            vid = EXCLUDED.vid
            RETURNING id"#,
                article_id.id,
                article_video_model.item_id,
                article_video_model.video_id,
                article_video_model.src,
                article_video_model.height,
                article_video_model.width,
                article_video_model.length,
                article_video_model.vid
            )
            .fetch_one(&*pool)
            .map_err(|e| {
                error!("Failed to insert article videos. Error: {e:?}");
                Error::Db("Failed to insert article videos.".to_string())
            })
            .await?;
        }

        let article_image_models =
            convert_article_to_article_image_models(article.clone(), article_id.id)?;

        for article_image_model in article_image_models {
            let _ = sqlx::query!(
                r#"
            INSERT INTO pocket_article_images (
                pocket_article_id, 
                item_id, 
                image_id, 
                src,
                width,
                height,
                caption,
                credit
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8
            )
            ON CONFLICT (
                pocket_article_id, 
                item_id, 
                image_id
            ) 
            DO UPDATE
            SET 
            src = EXCLUDED.src,
            width = EXCLUDED.width, 
            height = EXCLUDED.height, 
            caption = EXCLUDED.caption, 
            credit = EXCLUDED.credit
            RETURNING id"#,
                article_id.id,
                article_image_model.item_id,
                article_image_model.image_id,
                article_image_model.src,
                article_image_model.width,
                article_image_model.height,
                article_image_model.caption,
                article_image_model.credit
            )
            .fetch_one(&*pool)
            .map_err(|e| {
                error!("Failed to insert article images. Error: {e:?}");
                Error::Db("Failed to insert article images.".to_string())
            })
            .await?;
        }

        let article_author_models =
            convert_article_to_article_author_models(article, article_id.id)?;

        for article_author_model in article_author_models {}
    }

    Ok(())
}

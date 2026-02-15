use axum::{
    Json,
    extract::{Extension, Path, rejection::PathRejection},
    http::{StatusCode, header::HeaderMap},
    response::{Html, IntoResponse},
};
use chrono::Utc;
use sqlx::query_as;
use sqlx::{query, sqlite::SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tera::{Context, Tera};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::model::{
    CreatedTemporaryUrlResponse, GenarateUrlSecondsPayload, IssuedTemporaryUrls, TemporaryUrl,
    WikiTempDataTitleAndBody,
};
use crate::{error::AppError, utils::vec_to_hashmap};

// 一時URLの発行
pub async fn generate_url_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
    Json(payload): Json<GenarateUrlSecondsPayload>,
) -> Result<Json<CreatedTemporaryUrlResponse>, AppError> {
    // 現在時刻を取得
    let now = Utc::now().naive_utc();

    // 既に一時URLが発行されているWikiか確認
    let record = query!(
        r#"
        SELECT COUNT(*) as count
        FROM temporary_urls
        WHERE wiki_id = $1
        "#,
        wiki_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    if record.count > 0 {
        let _result = query!(
            r#"
            DELETE FROM temporary_urls
            WHERE wiki_id = $1
            "#,
            wiki_id
        )
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;
    }

    // WikiをユーザーIDとWikiのIDから取得
    let wiki = query_as!(
        WikiTempDataTitleAndBody,
        r#"
        SELECT
            id,
            title,
            body
        FROM wiki_model
        WHERE id = $1 AND user_id = $2
        "#,
        wiki_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    let uuid = Uuid::new_v4().to_string().to_string();
    let url = format!("/onetime/{}", uuid);
    let temp_url = TemporaryUrl::new(
        uuid,
        user_id,
        wiki.id.clone(),
        url,
        Duration::from_secs(payload.minutes * 60),
        wiki.title.clone(),
        wiki.body.clone(),
        now.to_string(),
    )
    .map_err(|_e| {
        return AppError::InternalServerError;
    })?;

    let created_url_response = query_as!(
        CreatedTemporaryUrlResponse,
        r#"
        INSERT INTO temporary_urls (
            id,
            user_id,
            wiki_id,
            url,
            expiration,
            title,
            body,
            create_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, url, expiration, title
        "#,
        temp_url.id,
        temp_url.user_id,
        wiki.id,
        temp_url.url,
        temp_url.expiration,
        wiki.title,
        wiki.body,
        now,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    Ok(Json(created_url_response))
}

// 一時URLからWikiを取得
pub async fn temporary_wiki_get_handler(
    headers: HeaderMap,
    Extension(pool): Extension<SqlitePool>,
    Extension(tera): Extension<Arc<Mutex<Tera>>>,
    url_id: Result<Path<String>, PathRejection>,
) -> Result<impl IntoResponse, AppError> {
    // User-Agentの取り出し
    let user_agent = headers.get("User-Agent").and_then(|ua| ua.to_str().ok());

    // User-Agentにmobileが含まれているか確認
    let is_mobile = user_agent.map_or(false, |ua| ua.contains("Mobile"));

    match url_id {
        // 正常な UUID が渡された場合
        Ok(Path(url_id)) => {
            let render_html = if is_mobile {
                "preview-mobile.html"
            } else {
                "preview.html"
            };

            let temp_url = query_as!(
                TemporaryUrl,
                r#"
                SELECT * FROM temporary_urls
                WHERE id = $1
                "#,
                url_id
            )
            .fetch_one(&pool)
            .await;

            match temp_url {
                Ok(temp_url) => {
                    // 共有URLが期限切れの場合
                    if temp_url.is_expired() {
                        query!(
                            r#"
                            DELETE FROM temporary_urls
                            WHERE id = $1
                            "#,
                            url_id
                        )
                        .execute(&pool)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;

                        let statuscode = "Not Found".to_string();
                        let message = "コンテンツが見つかりません。共有の期限切れやURLの入力間違いの可能性があります。".to_string();

                        let mut context = Context::new();
                        context.insert("statuscode", &statuscode);
                        context.insert("message", &message);

                        let tera = tera.lock().await;
                        match tera.render("notfound.html", &context) {
                            Ok(renderd) => return Ok(Html(renderd).into_response()),
                            Err(e) => {
                                tracing::error!("{}", e);
                                return Err(AppError::InternalServerError);
                            },
                        }

                    // 正常に 共有URLを返却できる場合
                    } else {
                        let title = temp_url.title;
                        let body = temp_url.body;

                        let mut context = Context::new();
                        context.insert("markdowntitle", &title);
                        context.insert("markdownbody", &body);

                        let tera = tera.lock().await;
                        match tera.render(render_html, &context) {
                            Ok(renderd) => Ok(Html(renderd).into_response()),
                            Err(e) => {
                                tracing::error!("{}", e);
                                return Err(AppError::InternalServerError);
                            },
                        }
                    }
                },
                // DBから共有URLの取得に失敗した場合
                Err(_) => {
                    let statuscode = "Not Found".to_string();
                    let message = "コンテンツが見つかりません。共有の期限切れやURLの入力間違いの可能性があります。".to_string();

                    let mut context = Context::new();
                    context.insert("statuscode", &statuscode);
                    context.insert("message", &message);

                    let tera = tera.lock().await;
                    match tera.render("notfound.html", &context) {
                        Ok(renderd) => return Ok(Html(renderd).into_response()),
                        Err(e) => {
                            tracing::error!("{}", e);
                            return Err(AppError::InternalServerError);
                        },
                    }
                },
            }
        },

        // 不正な UUID が渡された場合
        Err(_rejection) => {
            let statuscode = "Not Found".to_string();
            let message =
                "コンテンツが見つかりません。共有の期限切れやURLの入力間違いの可能性があります。"
                    .to_string();

            let mut context = Context::new();
            context.insert("statuscode", &statuscode);
            context.insert("message", &message);

            let tera = tera.lock().await;
            match tera.render("notfound.html", &context) {
                Ok(renderd) => return Ok(Html(renderd).into_response()),
                Err(e) => {
                    tracing::error!("{}", e);
                    return Err(AppError::InternalServerError);
                },
            }
        },
    }
}

// 一時URLの削除
pub async fn invalidate_url_handler(
    Path(url_id): Path<String>,
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<StatusCode, AppError> {
    query!(
        r#"
        DELETE FROM temporary_urls
        WHERE id = $1 AND user_id = $2
        "#,
        url_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|_| AppError::InternalServerError)?;

    Ok(StatusCode::NO_CONTENT)
}

// 発行済み一時URL一覧の取得
pub async fn get_all_temporary_urls(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, IssuedTemporaryUrls>>, AppError> {
    let urls = query_as!(
        IssuedTemporaryUrls,
        r#"
        SELECT
            id,
            user_id,
            wiki_id,
            url,
            expiration,
            title,
            create_at
        FROM temporary_urls
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    let urls_hash_map = vec_to_hashmap(urls, |u| u.id.clone());
    Ok(Json(urls_hash_map))
}

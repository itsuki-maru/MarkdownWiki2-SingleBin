use axum::{
    extract::{Path, Extension},
    response::{IntoResponse, Html},
    http::{StatusCode, header::HeaderMap}, Json,
};
use chrono::Utc;
use tera::{Context, Tera};
use sqlx::{sqlite::SqlitePool, query};
use sqlx::query_as;
use uuid::Uuid;
use std::time::Duration;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use super::super::scheme::{
    CreatedTemporaryUrlResponse,
    WikiTempDataTitleAndBody,
    TemporaryUrl,
    GenarateUrlSecondsPayload,
    IssuedTemporaryUrls
};
use super::super::custom_responses::custom_error_response;

// 一時URLの発行
pub async fn generate_url_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
    Json(payload): Json<GenarateUrlSecondsPayload>
) -> Result<Json<CreatedTemporaryUrlResponse>, impl IntoResponse> {

    // 現在時刻を取得
    let now = Utc::now().naive_utc();

    // 既に一時URLが発行されているWikiか確認
    let result = query!(
        "SELECT COUNT(*) as count FROM temporary_urls
        WHERE wiki_id = $1",
        wiki_id,
        
    )
    .fetch_one(&pool)
    .await;

    // 作成されていれば削除処理
    match result {
        Ok(row) => {
            if row.count > 0 {
                let _result = query!(
                    "DELETE FROM temporary_urls WHERE wiki_id = $1",
                    wiki_id
                )
                    .execute(&pool)
                    .await;                
            }
        },
        Err(_) => return Err(StatusCode::BAD_REQUEST)
    }

    // WikiをユーザーIDとWikiのIDから取得
    let result = query_as!(
        WikiTempDataTitleAndBody,
        "SELECT id, title, body FROM wiki_model WHERE id = $1 AND user_id = $2",
        wiki_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_e| {
        custom_error_response(
            "Database query wiki failed.",
            StatusCode::INTERNAL_SERVER_ERROR
        )
    });

    match result {
        Ok(wiki) => {
            let uuid = Uuid::new_v4().to_string();
            let url = format!("/onetime/{}", uuid);
            let temp_url = TemporaryUrl::new(
                uuid,
                user_id,
                wiki.id,
                url,
                Duration::from_secs(payload.minutes * 60),
                wiki.title.clone(),
                wiki.body.clone(),
                now.to_string(),
            );

            match temp_url {
                Ok(temp_url) => {
                    let temp_url_expiration = temp_url.expiration.clone().to_string();
                    let now_string = now.to_string();
                    let result = query_as!(
                        CreatedTemporaryUrlResponse,
                        "INSERT INTO temporary_urls (id, user_id, wiki_id, url, expiration, title, body, create_at)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                        RETURNING id, url, expiration, title",
                        temp_url.id,
                        temp_url.user_id,
                        wiki_id,
                        temp_url.url,
                        temp_url_expiration,
                        wiki.title,
                        wiki.body,
                        now_string,
                    )
                    .fetch_one(&pool)
                    .await;


                    match result {
                        Ok(created_url_response) => {
                            Ok(Json(created_url_response))
                        },
                        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                },
                Err(_) => return Err(StatusCode::BAD_REQUEST)
            }
        }
        Err(_) => return Err(StatusCode::BAD_REQUEST)
    }
}

// 一時URLからWikiを取得
pub async fn temporary_wiki_get_handler(
    headers: HeaderMap,
    Extension(pool): Extension<SqlitePool>,
    Extension(tera): Extension<Arc<Mutex<Tera>>>,
    Path(url_id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {

    // User-Agentの取り出し
    let user_agent = headers.get("User-Agent").and_then(|ua| ua.to_str().ok());

    // User-Agentにmobileが含まれているか確認
    let is_mobile = user_agent.map_or(false, |ua| ua.contains("Mobile"));

    let temp_url = query_as!(
        TemporaryUrl,
        "SELECT * FROM temporary_urls WHERE id = $1",
        url_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    if temp_url.is_expired() {
        let title = temp_url.title;
        query!("DELETE FROM temporary_urls WHERE id = $1", url_id)
            .execute(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let mut context = Context::new();
        context.insert("wikititle", &title);

        let tera = tera.lock().await;
        match tera.render("notfound.html", &context) {
            Ok(renderd) => return Ok(Html(renderd).into_response()),
            Err(e) => {
                tracing::error!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let title = temp_url.title;
    let body = temp_url.body;

    let mut context = Context::new();
    context.insert("markdowntitle", &title);
    context.insert("markdownbody", &body);

    // User-Agentで返却するHTMLを切り替え
    if is_mobile {
        let tera = tera.lock().await;
        match tera.render("preview-mobile.html", &context) {
            Ok(renderd) => Ok(Html(renderd).into_response()),
            Err(e) => {
                tracing::error!("{}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        let tera = tera.lock().await;
        match tera.render("preview.html", &context) {
            Ok(renderd) => Ok(Html(renderd).into_response()),
            Err(e) => {
                tracing::error!("{}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

// 一時URLの削除
pub async fn invalidate_url_handler(
    Path(url_id): Path<String>,
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<StatusCode, StatusCode> {

    query!("DELETE FROM temporary_urls WHERE id = $1 AND user_id = $2", url_id, user_id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// 発行済み一時URL一覧の取得
pub async fn get_all_temporary_urls(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, IssuedTemporaryUrls>>, impl IntoResponse> {

    let result = query_as!(
        IssuedTemporaryUrls,
        "SELECT id, user_id, wiki_id, url, expiration, title, create_at FROM temporary_urls
        WHERE user_id = $1",
        user_id,
    )
    .fetch_all(&pool)
    .await;

    let urls = match result {
        Ok(urls) => urls,
        Err(_) => {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let mut urls_hash_map = HashMap::new();
    for url in urls {
        let url_id = url.id.clone();
        urls_hash_map.insert(url_id, url);
    }
    Ok(Json(urls_hash_map))

}
use axum::{
    http::StatusCode,
    response::IntoResponse,
    response::Response, Json,
    extract::{Extension, Path, Query},
};
use chrono::Utc;
use serde_json::json;
use sqlx::sqlite::SqlitePool;
use sqlx::{query, query_as, Error as SqlxError};
use std::collections::HashMap;
use uuid::Uuid;
use super::super::custom_responses::custom_error_response;
use super::super::scheme::{
    CreateWikiData, DownloadWikiData, WikiData, ReturningId, WikiOwner, ResponseWikiData,
    ResponseWikiId, UpdateWikiData, UpdatedWikiResponse, WikiQueryParams,
};

// CREATE WIKI
pub async fn create_wiki_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<CreateWikiData>,
) -> Result<Json<ResponseWikiId>, impl IntoResponse> {

    // UTCで現在時刻を取得し、NaiveDateTimeに変換
    let now = Utc::now().naive_utc();

    // 新規WikiのID
    let new_wiki_id = Uuid::now_v7().to_string();

    let rec = query_as!(
        ReturningId,
        "INSERT INTO wiki_model (id, user_id, date, title, body, create_at, update_at, is_public)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        new_wiki_id,
        user_id,
        now,
        payload.title,
        payload.body,
        now,
        now,
        payload.is_public
    )
    .fetch_one(&pool)
    .await;

    match rec {
        Ok(wiki_id) => Ok(Json(ResponseWikiId {
            message: "New wiki created.".to_string(),
            user_id: user_id,
            new_wiki_id: wiki_id.id,
            date: now.to_string(),
        })),
        Err(_) => Err(custom_error_response(
            "Internal Server Error.",
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

// GET WIKI
pub async fn get_wiki_by_id_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<Json<WikiData>, StatusCode> {

    let result = query!("SELECT * FROM wiki_model WHERE id = $1", wiki_id)
        .fetch_one(&pool)
        .await;

    // データベースからの応答に基づいて処理
    match result {
        Ok(wiki) => Ok(Json(WikiData {
            id: wiki.id,
            user_id: wiki.user_id,
            date: wiki.date,
            title: wiki.title,
            body: wiki.body,
            update_at: wiki.update_at,
            is_public: wiki.is_public,
        })),
        Err(e) => match e {
            SqlxError::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

// GET WIKIS
pub async fn get_all_wiki_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, ResponseWikiData>>, impl IntoResponse> {

    let result = query_as!(
        WikiData,
        "SELECT id, user_id, date, title, body, update_at, is_public FROM wiki_model
        WHERE user_id = $1 OR is_public = true",
        user_id,
    )
    .fetch_all(&pool)
    .await;

    let memories = match result {
        Ok(memories) => memories,
        Err(_) => {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let mut wiki_hash_map = HashMap::new();
    for wiki in memories {
        let wiki_id = wiki.id.clone();
        let parsed_wiki = ResponseWikiData {
            id: wiki.id,
            user_id: wiki.user_id,
            date: wiki.date,
            title: wiki.title,
            body: wiki.body,
            update_at: wiki.update_at,
            is_public: wiki.is_public,
        };
        wiki_hash_map.insert(wiki_id, parsed_wiki);
    }
    Ok(Json(wiki_hash_map))
}

// GET WIKIS LIMIT
pub async fn get_wiki_limit_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(limit): Path<i64>,
) -> Result<Json<HashMap<String, WikiData>>, impl IntoResponse> {

    let result = query_as!(
        WikiData,
        "SELECT id, user_id, date, title, body, update_at, is_public FROM wiki_model
        WHERE user_id = $1 OR is_public = true ORDER BY id DESC LIMIT $2",
        user_id,
        limit,
    )
    .fetch_all(&pool)
    .await;

    let memories = match result {
        Ok(memories) => memories,
        Err(_) => {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let mut wiki_hash_map = HashMap::new();
    for wiki in memories {
        let wiki_id = wiki.id.clone();
        wiki_hash_map.insert(wiki_id, wiki);
    }

    Ok(Json(wiki_hash_map))
}

// GET WIKI OWNER
pub async fn get_wiki_owner_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    let result = query_as!(
        WikiOwner,
        "SELECT user_model.id, user_model.username, user_model.public_name FROM wiki_model
        JOIN user_model ON wiki_model.user_id = user_model.id
        WHERE (wiki_model.id = $1 AND wiki_model.user_id = $2)
        OR (wiki_model.id = $1 AND wiki_model.is_public = true)",
        wiki_id,
        user_id,
    )
    .fetch_one(&pool)
    .await;

    let owner = match result {
        Ok(owner) => owner,
        Err(_) => {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let mut is_owner = false;
    if user_id == owner.id {
        is_owner = true;
    }

    let response_data = json!({
        "WikiOwner": owner.username,
        "public_name": owner.public_name,
        "is_owner": is_owner,
    });

    Ok(Json(response_data))
}

// UPDATE WIKI
pub async fn update_wiki_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
    Json(payload): Json<UpdateWikiData>,
) -> Result<Json<UpdatedWikiResponse>, impl IntoResponse> {

    let now = Utc::now().naive_utc();

    let result = query_as!(
        ReturningId,
        "UPDATE wiki_model SET title=$1, body=$2, update_at=$3, is_public=$4
        WHERE id = $5 AND user_id = $6 RETURNING id",
        payload.title,
        payload.body,
        now,
        payload.is_public,
        wiki_id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(id) => Ok(Json(UpdatedWikiResponse {
            id: id.id,
            message: "Update Ok.".to_string(),
        })),
        Err(_) => Err(custom_error_response(
            "Internal Server Error.",
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

// DELETE WIKI
pub async fn delete_wiki_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    let result = query_as!(
        ReturningId,
        "DELETE FROM wiki_model WHERE id = $1
        AND user_id = $2 RETURNING id",
        wiki_id,
        user_id,
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(id) => Ok(Json(UpdatedWikiResponse {
            id: id.id,
            message: "Delete Ok.".to_string(),
        })),
        Err(_) => Err(custom_error_response(
            "Internal Server Error.",
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

// QUERY WIKI
pub async fn wiki_query_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<WikiQueryParams>,
) -> Result<Json<HashMap<String, WikiData>>, impl IntoResponse> {

    let query1 = params.query1;
    let query2 = params.query2;

    if query1 == "".to_string() && query2 == "".to_string() {
        let result = query_as!(
            WikiData,
            "SELECT id, user_id, date, title, body, update_at, is_public FROM wiki_model
            WHERE user_id = $1 OR is_public = true ORDER BY id DESC LIMIT 100",
            user_id,
        )
        .fetch_all(&pool)
        .await;

        let memories = match result {
            Ok(memories) => memories,
            Err(_) => {
                return Err(custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };

        let mut wiki_hash_map = HashMap::new();
        for wiki in memories {
            let wiki_id = wiki.id.clone();
            wiki_hash_map.insert(wiki_id, wiki);
        }

        Ok(Json(wiki_hash_map))
    } else if query2 == "".to_string() {
        let query_text = format!("%\\{}%", query1);
        let result = query_as!(
            WikiData,
            "SELECT id, user_id, date, title, body, update_at, is_public FROM wiki_model
            WHERE (user_id = $1 OR is_public = true)
            AND (title LIKE $2 ESCAPE '\\' OR body LIKE $2 ESCAPE '\\')
            ORDER BY id DESC",
            user_id,
            query_text,
        )
        .fetch_all(&pool)
        .await;

        let memories = match result {
            Ok(memories) => memories,
            Err(_) => {
                return Err(custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };

        let mut wiki_hash_map = HashMap::new();
        for wiki in memories {
            let wiki_id = wiki.id.clone();
            wiki_hash_map.insert(wiki_id, wiki);
        }

        Ok(Json(wiki_hash_map))
    } else if query1 == "".to_string() {
        let query_text = format!("%\\{}%", query2);
        let result = query_as!(
            WikiData,
            "SELECT id, user_id, date, title, body, update_at, is_public FROM wiki_model
            WHERE (user_id = $1 OR is_public = true)
            AND (title LIKE $2 ESCAPE '\\' OR body LIKE $2 ESCAPE '\\')
            ORDER BY id DESC",
            user_id,
            query_text,
        )
        .fetch_all(&pool)
        .await;

        let memories = match result {
            Ok(memories) => memories,
            Err(_) => {
                return Err(custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };

        let mut wiki_hash_map = HashMap::new();
        for wiki in memories {
            let wiki_id = wiki.id.clone();
            wiki_hash_map.insert(wiki_id, wiki);
        }

        Ok(Json(wiki_hash_map))
    } else {
        let query_text1 = format!("%\\{}%", query1);
        let query_text2 = format!("%\\{}%", query2);
        let result = query_as!(
            WikiData,
            "SELECT id, user_id, date, title, body, update_at, is_public FROM wiki_model
            WHERE (user_id = $1 OR is_public = true)
            AND (title LIKE $2 ESCAPE '\\' OR body LIKE $2 ESCAPE '\\')
            AND (title LIKE $3 ESCAPE '\\' OR body LIKE $3 ESCAPE '\\')
            ORDER BY id DESC",
            user_id,
            query_text1,
            query_text2,
        )
        .fetch_all(&pool)
        .await;

        let memories = match result {
            Ok(memories) => memories,
            Err(_) => {
                return Err(custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };

        let mut wiki_hash_map = HashMap::new();
        for wiki in memories {
            let wiki_id = wiki.id.clone();
            wiki_hash_map.insert(wiki_id, wiki);
        }

        Ok(Json(wiki_hash_map))
    }
}

// DOWNLOAD WIKI
pub async fn download_file(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    let result = query_as!(
        DownloadWikiData,
        "SELECT title, body FROM wiki_model
        WHERE id = $1 AND user_id = $2 OR id = $1 AND is_public = true",
        wiki_id,
        user_id,
    )
    .fetch_one(&pool)
    .await;

    let markdown_text = match result {
        Ok(markdown_data) => {
            let title = markdown_data.title;
            let body = markdown_data.body;
            format!("# {}\n\n{}", title, body)
        }
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    let response = Response::builder()
        .status(200)
        .header("Content-Type", "text/markdown")
        .header(
            "Content-Disposition",
            "attachment; filename=\"download.md\"",
        )
        .body(markdown_text)
        .map_err(|_e| {
            custom_error_response(
                "Failed to create response body.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        });

    Ok(response)
}

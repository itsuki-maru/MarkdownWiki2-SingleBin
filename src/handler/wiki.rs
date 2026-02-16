use crate::error::AppError;
use crate::model::{
    CreateWikiData, DownloadWikiData, ResponseWikiId, ReturningId, UpdateWikiData,
    UpdatedWikiResponse, WikiData, WikiOwner, WikiQueryParams,
};
use crate::utils::vec_to_hashmap;
use axum::{
    Json,
    extract::{Extension, Path, Query},
    response::IntoResponse,
    response::Response,
};
use chrono::Utc;
use serde_json::json;
use sqlx::query_as;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

// CREATE WIKI
pub async fn create_wiki_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<CreateWikiData>,
) -> Result<Json<ResponseWikiId>, AppError> {
    // UTCで現在時刻を取得し、NaiveDateTimeに変換
    let now = Utc::now().naive_utc();

    // 新規WikiのID
    let new_wiki_id = Uuid::now_v7().to_string();

    let new_wiki_id = query_as!(
        ReturningId,
        r#"
        INSERT INTO wiki_model (
            id,
            user_id,
            date,
            title,
            body,
            create_at,
            update_at,
            is_public
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
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
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    Ok(Json(ResponseWikiId {
        message: "New wiki created.".to_string(),
        user_id: user_id,
        new_wiki_id: new_wiki_id.id,
        date: now.to_string(),
    }))
}

// GET WIKI
pub async fn get_wiki_by_id_handler(
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<Json<WikiData>, AppError> {
    let wiki = query_as!(
        WikiData,
        r#"
        SELECT
            id,
            user_id,
            date,
            title,
            body,
            update_at,
            is_public,
            is_edit_request
        FROM wiki_model
        WHERE id = $1
        "#,
        wiki_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    Ok(Json(WikiData {
        id: wiki.id,
        user_id: wiki.user_id,
        date: wiki.date,
        title: wiki.title,
        body: wiki.body,
        update_at: wiki.update_at,
        is_public: wiki.is_public,
        is_edit_request: wiki.is_edit_request,
    }))
}

// GET WIKIS
pub async fn get_all_wiki_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, WikiData>>, AppError> {
    let wikis = query_as!(
        WikiData,
        r#"
        SELECT
            id,
            user_id,
            date,
            title,
            body,
            update_at,
            is_public,
            is_edit_request
        FROM wiki_model
        WHERE user_id = $1 OR is_public = true OR is_edit_request = true
        "#,
        user_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    let wiki_hash_map = vec_to_hashmap(wikis, |w| w.id.clone());
    Ok(Json(wiki_hash_map))
}

// GET WIKIS LIMIT
pub async fn get_wiki_limit_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(limit): Path<i64>,
) -> Result<Json<HashMap<String, WikiData>>, AppError> {
    let wikis = query_as!(
        WikiData,
        r#"
        SELECT
            id,
            user_id,
            date,
            title,
            body,
            update_at,
            is_public,
            is_edit_request
        FROM wiki_model
        WHERE user_id = $1 OR is_public = true OR is_edit_request = true
        ORDER BY id DESC LIMIT $2
        "#,
        user_id,
        limit,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    let wiki_hash_map = vec_to_hashmap(wikis, |w| w.id.clone());
    Ok(Json(wiki_hash_map))
}

// GET WIKI OWNER
pub async fn get_wiki_owner_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let owner = query_as!(
        WikiOwner,
        r#"
        SELECT
            user_model.id,
            user_model.username,
            user_model.public_name
        FROM wiki_model
        JOIN user_model ON wiki_model.user_id = user_model.id
        WHERE (wiki_model.id = $1 AND wiki_model.user_id = $2)
        OR (wiki_model.id = $1 AND wiki_model.is_public = true)
        "#,
        wiki_id,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

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
) -> Result<Json<UpdatedWikiResponse>, AppError> {
    let now = Utc::now().naive_utc();

    let returned_id = query_as!(
        ReturningId,
        r#"
        UPDATE wiki_model
        SET title=$1, body=$2, update_at=$3, is_public=$4
        WHERE id = $5 AND user_id = $6
        RETURNING id
        "#,
        payload.title,
        payload.body,
        now,
        payload.is_public,
        wiki_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    Ok(Json(UpdatedWikiResponse {
        id: returned_id.id,
        message: "Update Ok.".to_string(),
    }))
}

// DELETE WIKI
pub async fn delete_wiki_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let returned_id = query_as!(
        ReturningId,
        r#"
        DELETE FROM wiki_model
        WHERE id = $1
        AND user_id = $2
        RETURNING id
        "#,
        wiki_id,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    Ok(Json(UpdatedWikiResponse {
        id: returned_id.id,
        message: "Delete Ok.".to_string(),
    }))
}

// QUERY WIKI
pub async fn wiki_query_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<WikiQueryParams>,
) -> Result<Json<HashMap<String, WikiData>>, AppError> {
    let query1 = params.query1;
    let query2 = params.query2;

    if query1 == "".to_string() && query2 == "".to_string() {
        let wikis = query_as!(
            WikiData,
            r#"
            SELECT
                id,
                user_id,
                date,
                title,
                body,
                update_at,
                is_public,
                is_edit_request
            FROM wiki_model
            WHERE user_id = $1 OR is_public = true
            ORDER BY id DESC LIMIT 100
            "#,
            user_id,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        let wiki_hash_map = vec_to_hashmap(wikis, |w| w.id.clone());
        Ok(Json(wiki_hash_map))
    } else if query2 == "".to_string() {
        let query = format!("%\\{}%", query1);
        let wikis = query_as!(
            WikiData,
            "SELECT
                id,
                user_id,
                date,
                title,
                body,
                update_at,
                is_public,
                is_edit_request
            FROM wiki_model
            WHERE (user_id = $1 OR is_public = true)
            AND (title LIKE $2 ESCAPE '\\' OR body LIKE $2 ESCAPE '\\')
            ORDER BY id DESC
            ",
            user_id,
            query,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        let wiki_hash_map = vec_to_hashmap(wikis, |w| w.id.clone());
        Ok(Json(wiki_hash_map))
    } else if query1 == "".to_string() {
        let query = format!("%\\{}%", query2);
        let wikis = query_as!(
            WikiData,
            "SELECT
                id,
                user_id,
                date,
                title,
                body,
                update_at,
                is_public,
                is_edit_request
            FROM wiki_model
            WHERE (user_id = $1 OR is_public = true)
            AND (title LIKE $2 ESCAPE '\\' OR body LIKE $2 ESCAPE '\\')
            ORDER BY id DESC
            ",
            user_id,
            query,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        let wiki_hash_map = vec_to_hashmap(wikis, |w| w.id.clone());
        Ok(Json(wiki_hash_map))
    } else {
        let query1 = format!("%\\{}%", query1);
        let query2 = format!("%\\{}%", query2);
        let wikis = query_as!(
            WikiData,
            "SELECT
                id,
                user_id,
                date,
                title,
                body,
                update_at,
                is_public,
                is_edit_request
                FROM
            wiki_model
            WHERE (user_id = $1 OR is_public = true)
            AND (title LIKE $2 ESCAPE '\\' OR body LIKE $2 ESCAPE '\\')
            AND (title LIKE $3 ESCAPE '\\' OR body LIKE $3 ESCAPE '\\')
            ORDER BY id DESC
            ",
            user_id,
            query1,
            query2,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        let wiki_hash_map = vec_to_hashmap(wikis, |w| w.id.clone());
        Ok(Json(wiki_hash_map))
    }
}

// DOWNLOAD WIKI
pub async fn download_file(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let wiki = query_as!(
        DownloadWikiData,
        r#"
        SELECT
            title,
            body
        FROM wiki_model
        WHERE id = $1 AND user_id = $2
        OR id = $1 AND is_public = true
        "#,
        wiki_id,
        user_id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    let markdown_text = match wiki {
        Some(markdown_data) => {
            let title = markdown_data.title;
            let body = markdown_data.body;
            format!("# {}\n\n{}", title, body)
        },
        None => return Err(AppError::NotFound),
    };

    let response = Response::builder()
        .status(200)
        .header("Content-Type", "text/markdown")
        .header(
            "Content-Disposition",
            "attachment; filename=\"download.md\"",
        )
        .body(markdown_text)
        .map_err(|_e| AppError::InternalServerError)?;
    Ok(response)
}

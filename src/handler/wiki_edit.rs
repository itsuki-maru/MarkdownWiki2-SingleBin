use crate::error::AppError;
use crate::scheme::{
    EditResponseStatus, EditWikiListFromDb, EditWikiOwnerRequest, EditWikiOwnerResponse,
    EditWikiRequest, EditWikiStatusResponse, IsExists, ReturningId, WikiData,
};
use axum::{
    Json,
    extract::{Extension, Path},
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::json;
use sqlx::query_as;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

// Wikiの更新リクエスト
pub async fn request_wiki_edit(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(wiki_id): Path<String>,
    Json(payload): Json<EditWikiRequest>,
) -> Result<Json<EditWikiStatusResponse>, AppError> {
    // UTCで現在時刻を取得し、NaiveDateTimeに変換
    let now = Utc::now().naive_utc();

    // Wikiのオーナーを確認
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
        wiki_id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    let wiki = match wiki {
        Some(wiki) => wiki,
        None => return Err(AppError::NotFound),
    };

    // 変更リクエスト者とWikiのオーナーが同一の場合は何もしない
    if user_id == wiki.user_id {
        return Err(AppError::Validation("Your owner wiki.".into()));
    }

    // プライベートWikiの場合は何もしない
    if !wiki.is_public {
        return Err(AppError::Validation("Private wiki.".into()));
    }

    // すでにリクエスト済みであるか確認
    let edit_request_exists = query_as!(
        IsExists,
        r#"
        SELECT EXISTS(SELECT 1 FROM edit_request_wiki_model
        WHERE request_wiki_id = $1) as exists_flag
        "#,
        wiki_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // `i64` を `bool` に変換
    let edit_request_exists = edit_request_exists.exists_flag != 0;

    // 既にリクエスト済みのWikiであればエラーを返す
    if edit_request_exists {
        return Err(AppError::Conflict);
    }

    // トランザクション開始
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "failed to begin transaction");
        AppError::InternalServerError
    })?;

    // 新規編集リクエストWikiのID
    let new_edit_wiki_req_id = Uuid::now_v7().to_string();

    let status = payload.status.as_str();

    let new_edit_request_id = query_as!(
        ReturningId,
        r#"
        INSERT INTO edit_request_wiki_model (
            id,
            wiki_owner_id,
            request_user_id,
            request_wiki_id,
            edit_request_title,
            edit_request_body,
            create_at,
            request_message,
            status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        new_edit_wiki_req_id,
        wiki.user_id,
        user_id,
        wiki_id,
        payload.edit_request_title,
        payload.edit_request_body,
        now,
        payload.request_message,
        status,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // wiki_model を更新
    let _returned_id = query_as!(
        ReturningId,
        r#"
        UPDATE wiki_model
        SET is_edit_request = True
        WHERE id = $1
        RETURNING id
        "#,
        wiki_id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // トランザクション終了
    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "failed to commit transaction");
        AppError::Sqlx(e)
    })?;

    let status = EditWikiStatusResponse {
        id: new_edit_request_id.id,
        message: "Edit request ok.".into(),
        status: payload.status,
    };
    Ok(Json(status))
}

// 更新リクエスト中のWiki（オーナー側又は申請側）の一覧取得
pub async fn get_edit_request_wikis(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, EditWikiListFromDb>>, AppError> {
    // オーナーかリクエスト者のいずれかを取得
    let edit_request_wikis = query_as!(
        EditWikiListFromDb,
        r#"
        SELECT
            edit_request_wiki_model.id,
            wiki_owner_id,
            user_model.public_name as request_public_user_name,
            request_wiki_id,
            wiki_model.title as original_title,
            wiki_model.body as original_body,
            edit_request_title,
            edit_request_body,
            edit_request_wiki_model.create_at,
            edit_request_wiki_model.request_message,
            status
        FROM edit_request_wiki_model
        JOIN user_model ON edit_request_wiki_model.request_user_id = user_model.id
        JOIN wiki_model ON edit_request_wiki_model.request_wiki_id = wiki_model.id
        WHERE wiki_owner_id = $1 OR request_user_id = $1
        "#,
        user_id,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    let mut wiki_hash_map = HashMap::new();
    for wiki in edit_request_wikis {
        let wiki_id = wiki.id.clone();
        let parsed_wiki = EditWikiListFromDb {
            id: wiki.id,
            wiki_owner_id: wiki.wiki_owner_id,
            request_public_user_name: wiki.request_public_user_name,
            request_wiki_id: wiki.request_wiki_id,
            original_title: wiki.original_title,
            original_body: wiki.original_body,
            edit_request_title: wiki.edit_request_title,
            edit_request_body: wiki.edit_request_body,
            create_at: wiki.create_at,
            request_message: wiki.request_message,
            status: wiki.status,
        };
        wiki_hash_map.insert(wiki_id, parsed_wiki);
    }
    Ok(Json(wiki_hash_map))
}

// オーナーの承認・却下
pub async fn edit_request_owner_result(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<EditWikiOwnerRequest>,
) -> Result<Json<EditWikiOwnerResponse>, AppError> {
    // 更新リクエストWikiの取得
    let edit_wiki = query_as!(
        EditWikiListFromDb,
        r#"
        SELECT
            edit_request_wiki_model.id,
            wiki_owner_id,
            user_model.public_name as request_public_user_name,
            request_wiki_id,
            wiki_model.title as original_title,
            wiki_model.body as original_body,
            edit_request_title,
            edit_request_body,
            edit_request_wiki_model.create_at,
            edit_request_wiki_model.request_message,
            status
        FROM edit_request_wiki_model
        JOIN user_model ON edit_request_wiki_model.request_user_id = user_model.id
        JOIN wiki_model ON edit_request_wiki_model.request_wiki_id = wiki_model.id
        WHERE edit_request_wiki_model.id = $1 AND wiki_owner_id = $2
        "#,
        payload.id,
        user_id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error. get target wiki.");
        AppError::Sqlx(e)
    })?;

    let edit_wiki = match edit_wiki {
        Some(edit_wiki) => edit_wiki,
        None => return Err(AppError::NotFound),
    };

    // 変更対象のWikiのオーナーとアクセスしてきたユーザーが異なる場合はエラー
    if user_id != edit_wiki.wiki_owner_id {
        return Err(AppError::Unauthorized("Not owner request".into()));
    }

    // トランザクション開始
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "failed to begin transaction");
        AppError::InternalServerError
    })?;

    // 却下の場合
    if payload.reject {
        // edit_request_wiki_model テーブルを更新
        query_as!(
            ReturningId,
            r#"
            UPDATE edit_request_wiki_model
            SET status = 'REJECT'
            WHERE id = $1
            RETURNING id
            "#,
            payload.id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error. reject update edit_request_wiki_model.");
            AppError::Sqlx(e)
        })?;

        // wiki_model を更新（is_edit_requestフラグをfalse）
        let _returned_id = query_as!(
            ReturningId,
            r#"
            UPDATE wiki_model
            SET is_edit_request = False
            WHERE id = $1
            RETURNING id
            "#,
            edit_wiki.request_wiki_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error. reject update wiki_model.");
            AppError::Sqlx(e)
        })?;

        let status = EditWikiOwnerResponse {
            id: payload.id,
            status: EditResponseStatus::Reject,
        };

        // トランザクション終了
        tx.commit().await.map_err(|e| {
            tracing::error!(error = %e, "failed to commit transaction");
            AppError::Sqlx(e)
        })?;

        return Ok(Json(status));

    // 承認の場合
    } else {
        // UTCで現在時刻を取得し、NaiveDateTimeに変換
        let now = Utc::now().naive_utc();

        // wiki_model テーブルを更新（申請を反映し、is_edit_requestフラグをfalse）
        query_as!(
            ReturningId,
            r#"
            UPDATE wiki_model
            SET title = $1, body = $2, update_at = $3, is_edit_request = False
            WHERE id = $4
            RETURNING id
            "#,
            edit_wiki.edit_request_title,
            edit_wiki.edit_request_body,
            now,
            edit_wiki.request_wiki_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error. applied update wiki_model.");
            AppError::Sqlx(e)
        })?;

        // edit_request_wiki_model から削除
        query_as!(
            ReturningId,
            r#"
            DELETE FROM edit_request_wiki_model
            WHERE id = $1
            RETURNING id
            "#,
            payload.id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error. applied delete edit_request_wiki_model.");
            AppError::Sqlx(e)
        })?;

        let status = EditWikiOwnerResponse {
            id: payload.id,
            status: EditResponseStatus::Apply,
        };

        // トランザクション終了
        tx.commit().await.map_err(|e| {
            tracing::error!(error = %e, "failed to commit transaction");
            AppError::Sqlx(e)
        })?;

        Ok(Json(status))
    }
}

// 申請の取り下げ
pub async fn disable_edit_request(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(edit_request_wiki_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // edit_request_wiki_model から削除
    let query_result = query_as!(
        ReturningId,
        r#"
        DELETE FROM edit_request_wiki_model
        WHERE id = $1
        AND request_user_id = $2
        RETURNING id
        "#,
        edit_request_wiki_id,
        user_id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    match query_result {
        Some(_) => {}
        None => return Err(AppError::NotFound),
    };

    let response_data = json!({
        "deleted": "ok".to_string(),
    });

    Ok(Json(response_data))
}

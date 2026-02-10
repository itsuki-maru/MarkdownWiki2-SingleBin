use axum::{
    Json,
    extract::{Extension, Path},
    response::{Html, IntoResponse},
};
use bcrypt::{DEFAULT_COST, hash};

use crate::error::AppError;
use crate::scheme::{
    IsExists, IsSuperuser, ResponseUserData, ReturningId, SignupPayload, UpdateUserPasswordData,
    UpdateUserPublicNameData,
};
use chrono::{TimeDelta, Utc};
use rust_embed::RustEmbed;
use serde_json::json;
use sqlx::query_as;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

// 管理者画面の取得
pub async fn admin_index_get_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, AppError> {
    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        r#"
        SELECT is_superuser FROM user_model
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // 管理者アカウント権限を持ったユーザーの場合
    if super_user_exists.is_superuser {
        match Asset::get("index-admin.html") {
            Some(content) => {
                let html_content = String::from_utf8(content.data.into_owned()).unwrap();
                return Ok(Html(html_content));
            }
            None => return Err(AppError::NotFound),
        }
    // 管理者以外
    } else {
        match Asset::get("index.html") {
            Some(content) => {
                let html_content = String::from_utf8(content.data.into_owned()).unwrap();
                return Ok(Html(html_content));
            }
            None => return Err(AppError::NotFound),
        }
    }
}

// ユーザー一覧を取得
pub async fn get_users_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, ResponseUserData>>, AppError> {
    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        r#"
        SELECT is_superuser FROM user_model
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    if super_user_exists.is_superuser {
        let users = query_as!(
            ResponseUserData,
            r#"
            SELECT
                id,
                username,
                password,
                public_name,
                create_at,
                is_superuser,
                is_locked
            FROM user_model
            "#
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        let mut users_hash_map = HashMap::new();
        for user in users {
            let user_id = user.id.clone();
            let parsed_user = ResponseUserData {
                id: user.id,
                username: user.username,
                public_name: user.public_name,
                password: user.password,
                create_at: user.create_at,
                is_superuser: user.is_superuser,
                is_locked: user.is_locked,
            };
            users_hash_map.insert(user_id, parsed_user);
        }
        Ok(Json(users_hash_map))
    } else {
        Err(AppError::Unauthorized("Not admin user.".into()))
    }
}

// ユーザーパスワードの変更
pub async fn update_users_password_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(update_user_id): Path<String>,
    Json(payload): Json<UpdateUserPasswordData>,
) -> Result<impl IntoResponse, AppError> {
    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        r#"
        SELECT is_superuser FROM user_model
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // 管理者アカウント権限を持ったユーザーの場合
    if super_user_exists.is_superuser {
        let hashed_password = hash(payload.new_password, DEFAULT_COST).unwrap_or("".to_string());
        if &hashed_password == "" {
            return Err(AppError::InternalServerError);
        }
        query_as!(
            ReturningId,
            r#"
            UPDATE user_model
            SET password = $1
            WHERE id = $2
            RETURNING id
            "#,
            hashed_password,
            update_user_id,
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;
        let body = json!({"message": "Password Reset Ok."}).to_string();
        return Ok(body);

    // 管理者以外
    } else {
        Err(AppError::Unauthorized("Update Error.".into()))
    }
}

// ユーザー公開名更新ハンドラー
pub async fn update_public_name_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(update_user_id): Path<String>,
    Json(payload): Json<UpdateUserPublicNameData>,
) -> Result<impl IntoResponse, AppError> {
    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        r#"
        SELECT is_superuser FROM user_model
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // 管理者アカウント権限を持ったユーザーの場合
    if super_user_exists.is_superuser {
        query_as!(
            ReturningId,
            r#"
            UPDATE user_model
            SET public_name = $1
            WHERE id = $2
            RETURNING id
            "#,
            payload.public_name,
            update_user_id,
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        let body = json!({"message": "Public Name Update Ok."}).to_string();
        return Ok(body);

    // 管理者以外
    } else {
        Err(AppError::Unauthorized("Update Error.".into()))
    }
}

// アカウントロック解除ハンドラー
pub async fn unlock_account_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(unlock_user_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        r#"
        SELECT is_superuser FROM user_model
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // 管理者アカウント権限を持ったユーザーの場合
    if super_user_exists.is_superuser {
        query_as!(
            ReturningId,
            r#"
            UPDATE user_model
            SET is_locked = $1
            WHERE id = $2
            RETURNING id
            "#,
            false,
            unlock_user_id,
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;
        let body = json!({"message": "Unlock Ok."}).to_string();
        return Ok(body);

    // 管理者以外
    } else {
        Err(AppError::Unauthorized("Unlock Error.".into()))
    }
}

// ユーザー作成
pub async fn create_users_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<SignupPayload>,
) -> Result<Json<ReturningId>, AppError> {
    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        r#"
        SELECT is_superuser FROM user_model
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "database error.");
        AppError::Sqlx(e)
    })?;

    // 管理者アカウント権限を持ったユーザーの場合
    if super_user_exists.is_superuser {
        // 既に同名のユーザーが存在するか確認
        let user_exists = query_as!(
            IsExists,
            r#"
            SELECT EXISTS(SELECT 1 FROM user_model
            WHERE username = $1) as exists_flag
            "#,
            payload.username
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        // `i64`を`bool`に変換
        let user_exists = user_exists.exists_flag != 0;

        // 同名のユーザーが既に存在する場合はエラーを返す
        if user_exists {
            return Err(AppError::Conflict);
        }

        // パスワードをハッシュ化(ソルト値はハッシュ値に組み込んで管理)
        let hashed_password = hash(payload.password, DEFAULT_COST).unwrap_or("".to_string());
        if hashed_password == "" {
            return Err(AppError::InternalServerError);
        }

        // UTCで現在時刻を取得し、NaiveDateTimeに変換
        let now = Utc::now().naive_utc();
        let yesterday;
        match TimeDelta::try_days(1) {
            Some(one_day_delta) => {
                yesterday = now - one_day_delta;
            }
            None => {
                tracing::error!("Initial Data Create Error.");
                return Err(AppError::InternalServerError);
            }
        }

        // トランザクションの開始
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!(error = %e, "failed to begin transaction");
            AppError::InternalServerError
        })?;

        // 新規ID
        let new_user_id = Uuid::now_v7().to_string();
        let totp_secret = "".to_string();
        let totp_temp_secret = "".to_string();

        // 新しいユーザーを追加（追加したユーザーのidを取得）
        let new_user_id = query_as!(
            ReturningId,
            r#"
        INSERT INTO user_model (
            id,
            username,
            public_name,
            password,
            create_at,
            is_superuser,
            failed_count,
            next_challenge_time,
            is_locked,
            is_private,
            is_basic_authed,
            is_basic_authed_at,
            totp_secret,
            totp_temp_secret
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id
        "#,
            new_user_id,
            payload.username,
            payload.public_name,
            hashed_password,
            now,
            false,
            0,
            yesterday,
            false,
            true,
            false,
            yesterday,
            totp_secret,
            totp_temp_secret,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "database error.");
            AppError::Sqlx(e)
        })?;

        // トランザクションの終了
        tx.commit().await.map_err(|e| {
            tracing::error!(error = %e, "failed to commit transaction");
            AppError::Sqlx(e)
        })?;

        Ok(Json(new_user_id))

    // 管理者以外
    } else {
        Err(AppError::Unauthorized("Not Superuser.".into()))
    }
}

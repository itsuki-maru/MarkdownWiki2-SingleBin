use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Json, extract::{Extension, Path},
};
use bcrypt::{hash, DEFAULT_COST};

use chrono::{TimeDelta, Utc};
use serde_json::json;
use sqlx::sqlite::SqlitePool;
use sqlx::query_as;
use uuid::Uuid;
use std::collections::HashMap;
use super::super::custom_responses::custom_error_response;
use super::super::scheme::{ResponseUserData, UpdateUserPasswordData, UpdateUserPublicNameData, ReturningId, SignupPayload, IsSuperuser, IsExists};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

// ADMIN HTML GET
pub async fn admin_index_get_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, impl IntoResponse> {

    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        "SELECT is_superuser FROM user_model WHERE id = $1",
        user_id,
    )
    .fetch_one(&pool)
    .await;

    // `i64`を`bool`に変換
    let mut is_superuser = false;
    match super_user_exists {
        Ok(super_user) => {
            if super_user.is_superuser {
                is_superuser = true;
            }
        },
        Err(_e) => {}
    }

    if is_superuser {
        match Asset::get("index-admin.html") {
            Some(content) => {
                let html_content = String::from_utf8(content.data.into_owned()).unwrap();
                return Ok(Html(html_content))
            }
            None => return Err((StatusCode::NOT_FOUND, "Content not foubnd").into_response())
        }
    }
    match Asset::get("index.html") {
        Some(content) => {
            let html_content = String::from_utf8(content.data.into_owned()).unwrap();
            return Ok(Html(html_content))
        }
        None => return Err((StatusCode::NOT_FOUND, "Content not foubnd").into_response())
    }
}

// ユーザー一覧を取得
pub async fn get_users_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, ResponseUserData>>, impl IntoResponse> {

    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        "SELECT is_superuser FROM user_model WHERE id = $1",
        user_id,
    )
    .fetch_one(&pool)
    .await;

    // `i64`を`bool`に変換
    let mut is_superuser = false;
    match super_user_exists {
        Ok(super_user) => {
            if super_user.is_superuser {
                is_superuser = true;
            }
        },
        Err(_e) => {}
    }

    if is_superuser {
        let result = query_as!(
            ResponseUserData,
            "SELECT id, username, public_name, password, create_at, is_superuser, is_locked FROM user_model"
        )
        .fetch_all(&pool)
        .await;

        let users = match result {
            Ok(users) => users,
            Err(_) => {
                return Err(custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        };

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
        Err(custom_error_response(
            "Not admin user.",
            StatusCode::UNAUTHORIZED,
        ))
    }
}

// UPDATE USER PASSWORD
pub async fn update_users_password_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(update_user_id): Path<String>,
    Json(payload): Json<UpdateUserPasswordData>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        "SELECT is_superuser FROM user_model WHERE id = $1",
        user_id,
    )
    .fetch_one(&pool)
    .await;

    // `i64`を`bool`に変換
    let mut is_superuser = false;
    match super_user_exists {
        Ok(super_user) => {
            if super_user.is_superuser {
                is_superuser = true;
            }
        },
        Err(_e) => {}
    }

    if is_superuser {
        let hashed_password = hash(payload.new_password, DEFAULT_COST).unwrap_or("".to_string());
        if &hashed_password == "" {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        let result = query_as!(
            ReturningId,
            "UPDATE user_model SET password = $1 WHERE id = $2 RETURNING id",
            hashed_password,
            update_user_id,
        )
        .fetch_one(&pool)
        .await;

        match result {
            Ok(_) => {
                let body = json!({"message": "Password Reset Ok."}).to_string();
                return Ok(body);
            }
            Err(_) => {
                return Err(custom_error_response(
                    "Update Error.",
                    StatusCode::BAD_REQUEST,
                ))
            }
        }
    } else {
        Err(custom_error_response(
            "Update Error.",
            StatusCode::BAD_REQUEST,
        ))
    }
}

// UPDATE USER PASSWORD
pub async fn update_public_name_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(update_user_id): Path<String>,
    Json(payload): Json<UpdateUserPublicNameData>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        "SELECT is_superuser FROM user_model WHERE id = $1",
        user_id,
    )
    .fetch_one(&pool)
    .await;

    // `i64`を`bool`に変換
    let mut is_superuser = false;
    match super_user_exists {
        Ok(super_user) => {
            if super_user.is_superuser {
                is_superuser = true;
            }
        },
        Err(_e) => {}
    }

    if is_superuser {
        let result = query_as!(
            ReturningId,
            "UPDATE user_model SET public_name = $1 WHERE id = $2 RETURNING id",
            payload.public_name,
            update_user_id,
        )
        .fetch_one(&pool)
        .await;

        match result {
            Ok(_) => {
                let body = json!({"message": "Public Name Update Ok."}).to_string();
                return Ok(body);
            }
            Err(_) => {
                return Err(custom_error_response(
                    "Update Error.",
                    StatusCode::BAD_REQUEST,
                ))
            }
        }
    } else {
        Err(custom_error_response(
            "Update Error.",
            StatusCode::BAD_REQUEST,
        ))
    }
}

// アカウントロック解除ハンドラー
pub async fn unlock_account_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(unlock_user_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        "SELECT is_superuser FROM user_model WHERE id = $1",
        user_id,
    )
    .fetch_one(&pool)
    .await;

    // `i64`を`bool`に変換
    let mut is_superuser = false;
    match super_user_exists {
        Ok(super_user) => {
            if super_user.is_superuser {
                is_superuser = true;
            }
        },
        Err(_e) => {}
    }

    if is_superuser {
        let result = query_as!(
            ReturningId,
            "UPDATE user_model SET is_locked = $1 WHERE id = $2 RETURNING id",
            false,
            unlock_user_id,
        )
        .fetch_one(&pool)
        .await;

        match result {
            Ok(_) => {
                let body = json!({"message": "Unlock Ok."}).to_string();
                return Ok(body);
            }
            Err(_) => {
                return Err(custom_error_response(
                    "Unlock Error.",
                    StatusCode::BAD_REQUEST,
                ))
            }
        }
    } else {
        Err(custom_error_response(
            "Unlock Error.",
            StatusCode::BAD_REQUEST,
        ))
    }
}

// CREATE USER PASSWORD
pub async fn create_users_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<SignupPayload>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    // スーパーユーザー判定
    let super_user_exists = query_as!(
        IsSuperuser,
        "SELECT is_superuser FROM user_model WHERE id = $1",
        user_id,
    )
    .fetch_one(&pool)
    .await;

    // `i64`を`bool`に変換
    let mut is_superuser = false;
    match super_user_exists {
        Ok(super_user) => {
            if super_user.is_superuser {
                is_superuser = true;
            }
        },
        Err(_e) => {}
    }

    if is_superuser {
        // 既に同名のユーザーが存在するか確認
        let user_exists = query_as!(
            IsExists,
            "SELECT EXISTS(SELECT 1 FROM user_model
            WHERE username = $1) as exists_flag",
            payload.username
        )
        .fetch_one(&pool)
        .await
        .map_err(|_e| {
            custom_error_response("Database query failed.", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        // `i64`を`bool`に変換
        let user_exists = user_exists.exists_flag != 0;
        
        // 同名のユーザーが既に存在する場合はエラーを返す
        if user_exists {
            return Err(custom_error_response(
                "Conflict",
                StatusCode::CONFLICT,
            ));
        }
        
        // パスワードをハッシュ化(ソルト値はハッシュ値に組み込んで管理)
        let hashed_password = hash(payload.password, DEFAULT_COST).unwrap_or("".to_string());
        if hashed_password == "" {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        
        // UTCで現在時刻を取得し、NaiveDateTimeに変換
        let now = Utc::now().naive_utc();
        let yesterday;
        match TimeDelta::try_days(1) {
            Some(one_day_delta) => {
                yesterday = now - one_day_delta;
            },
            None => {
                tracing::error!("Initial Data Create Error.");
                panic!("Initial Data Create Error.");
            }
        }
        
        // 新規ID
        let new_user_id = Uuid::now_v7().to_string();
        let totp_secret = "".to_string();
        let totp_temp_secret = "".to_string();
        
        // ユーザーが存在しない場合は新しいユーザーを追加し、追加したユーザーのidを取得
        let rec = query_as!(
            ReturningId,
            "INSERT INTO user_model (id, username, public_name, password, create_at, is_superuser, failed_count, next_challenge_time, is_locked, is_private, is_basic_authed, is_basic_authed_at, totp_secret, totp_temp_secret)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING id",
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
        .fetch_one(&pool)
        .await;
    
        match rec {
            Ok(user_id) => Ok(Json(user_id)),
            Err(_) => Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    } else {
        Err(custom_error_response(
            "Not Superuser.",
            StatusCode::BAD_REQUEST,
        ))
    }
}

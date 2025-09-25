use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use serde_json::json;
use sqlx::sqlite::SqlitePool;
use sqlx::{query, query_as, Error as SqlxError};
use uuid::Uuid;
use crate::config::CONFIG;

use super::super::auth::{
    create_token,
    refresh_access_token,
};
use super::super::custom_responses::custom_error_response;
use super::super::scheme::{
    MessageApi,
    AuthenticatedUser,
    LoginPayload,
    SignupPayload,
    UserAccountModel,
    ReturningId,
    UpdateAccountPrivacyPayload,
    AccountPrivacyInfo,
    IsExists
};

// SIGNUP USER API
pub async fn signup_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<SignupPayload>,
) -> Result<Json<ReturningId>, impl IntoResponse> {
    
    // 既に同名のユーザーが存在するか確認
    let user_exists = query_as!(
        IsExists,
        "SELECT EXISTS(SELECT 1 FROM user_model WHERE username = $1) as exists_flag",
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
            "A user with the same name already exists.",
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
        "INSERT INTO user_model (id, username, password, create_at, is_superuser, failed_count, next_challenge_time, is_locked, is_private, is_basic_authed, is_basic_authed_at, totp_secret, totp_temp_secret)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id",
        new_user_id,
        payload.username,
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
        Ok(user_id) => return Ok(Json(user_id)),
        Err(_) => Err(custom_error_response(
            "Internal Server Error.",
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

// ログインハンドラー
pub async fn token_handler(
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    // application_settingsの値を格納する構造体
    struct ApplicationSettings {
        setting_key: String,
        setting_value: String,
    }

    let result_settings = query_as!(
        ApplicationSettings,
        "SELECT setting_key, setting_value FROM application_settings",
    )
    .fetch_all(&pool)
    .await;

    let mut parsed_login_limit = 15;
    let mut parsed_minutes= 5;
    let mut parsed_challenge_limit_start= 5;
    if let Ok(setting) = result_settings {
        for row in setting {
            if row.setting_key == "login_attempts_limit" {
                let login_attempts_limit = row.setting_value;
                parsed_login_limit = parsed_i64_to_string(login_attempts_limit).unwrap_or(15);

            } else if row.setting_key == "next_challenge_minutes" {
                let next_challenge_minutes = row.setting_value;
                parsed_minutes = parsed_i64_to_string(next_challenge_minutes).unwrap_or(5);

            } else if row.setting_key == "challenge_limit_start" {
                let challenge_limit_start = row.setting_value;
                parsed_challenge_limit_start = parsed_i64_to_string(challenge_limit_start).unwrap_or(5);
            }
        }
    }

    // ユーザー名からユーザーを取得
    let user = query_as!(
        UserAccountModel,
        "SELECT
            id,
            username,
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
        FROM user_model
        WHERE username = $1",
        payload.username
    )
    .fetch_one(&pool)
    .await
    .map_err(|_e| {
        custom_error_response(
            "Unauthorized",
            StatusCode::UNAUTHORIZED,
        )
    })?;

    // アカウントがロックされている場合はエラーレスポンス
    if user.is_locked {
        return Err(custom_error_response(
            "LockedAccount",
            StatusCode::UNAUTHORIZED,
        ));
    }

    // すでに設定回数以上失敗し、次にチャレンジできる時間に達していなければエラーレスポンス
    let current_datetime = Utc::now().naive_utc();
    // SQLiteでの文字列から日付型に戻す
    match parse_naive_datetime(&user.next_challenge_time) {
        Some(next) if next > current_datetime => {
            return Err(custom_error_response("PleaseWait", StatusCode::UNAUTHORIZED));
        }
        Some(_) => {}
        None => {
            return Err(custom_error_response("Parse Error.", StatusCode::INTERNAL_SERVER_ERROR));
        }
    }

    // ログイン失敗回数が上限に達している場合はアカウントをロックしてエラーレスポンス（カウントリセット）
    if user.failed_count == parsed_login_limit - 1 {
        let result = query!(
            "UPDATE user_model SET is_locked = $1, failed_count = $2  WHERE id = $3",
            true,
            0,
            user.id
        )
        .execute(&pool)
        .await;

        match result {
            Ok(_) => {
                return Err(custom_error_response(
                    "Locked",
                    StatusCode::UNAUTHORIZED,
                ))
            }
            Err(e) => {
                eprintln!("{}", e);
                return Err(custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    }

    // パスワード検証（ユーザー存在確認済）
    if verify(&payload.password, &user.password).map_err(|_e| {
        custom_error_response("Internal Server Error.", StatusCode::INTERNAL_SERVER_ERROR)
    })? == false
    {
    
        let failed_count = user.failed_count;
        let failed_count = failed_count + 1;

        // 失敗が設定回数に達したら次にチャレンジできる時間を設定
        if failed_count >= parsed_challenge_limit_start {
            let now = Utc::now().naive_utc();
            let five_minutes_later: NaiveDateTime;
            match TimeDelta::try_minutes(parsed_minutes.into()) {
                Some(five_min_delta) => {
                    five_minutes_later = now + five_min_delta;
                    let result = query!(
                        "UPDATE user_model SET failed_count = $1, next_challenge_time = $2 WHERE id = $3",
                        failed_count,
                        five_minutes_later,
                        user.id
                    )
                    .execute(&pool)
                    .await;

                    match result {
                        Ok(_) => {
                            return Err(custom_error_response(
                                "UnauthorizedPleaseWait",
                                StatusCode::UNAUTHORIZED,
                            ))
                        }
                        Err(e) => {
                            tracing::error!("{}", e);
                            return Err(custom_error_response(
                                "Internal Server Error.",
                                StatusCode::INTERNAL_SERVER_ERROR,
                            ));
                        }
                    }

                },
                None => {
                    tracing::error!("five_min_delta Get Error.");
                    return Err(custom_error_response(
                        "Internal Server Error.",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            }
        }

        // 認証に失敗したらカウントアップしエラーレスポンス
        let result = query!(
            "UPDATE user_model SET failed_count = $1 WHERE id = $2",
            failed_count,
            user.id
        )
        .execute(&pool)
        .await;

        match result {
            Ok(_) => {
                return Err(custom_error_response(
                    "Unauthorized",
                    StatusCode::UNAUTHORIZED,
                ))
            }
            Err(e) => {
                tracing::error!("{}", e);
                return Err(custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    }

    // TOTPが有効であれば要求
    if user.totp_secret != "" {
        let now = Utc::now().naive_utc().to_string();
        // ベーシック認証確認済みのフラグ
        query!("UPDATE user_model SET is_basic_authed = $1, is_basic_authed_at = $2 WHERE id = $3", true, now, user.id)
            .execute(&pool)
            .await
            .map_err(|_e| {
                custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        )?;
        let builder = Response::builder();
        let body = json!({
            "success": false,
            "user": payload.username,
            "id": user.id,
            "totp_required": true,
        }).to_string();
        // レスポンスの生成
        let response = builder
            .status(StatusCode::OK)
            .body(body)
            .map_err(|_e| {
                custom_error_response(
                    "Failed to create response body.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        )?;
        return Ok(response);
    // TOTPが有効でなければそのままログイン成功
    } else {
        // ログインに成功したらfailed_countをリセット
        query!("UPDATE user_model SET failed_count = $1 WHERE id = $2", 0, user.id)
            .execute(&pool)
            .await
            .map_err(|_e| {
                custom_error_response(
                    "Internal Server Error.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        )?;

        // アクセストークン生成
        let access_token = create_token(&user.id, CONFIG.access_token_exp_minutes, "access_token".to_string()).map_err(|_e| {
            custom_error_response("Failed to create token.", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        // リフレッシュトークン生成
        let refresh_token = create_token(&user.id, CONFIG.refresh_token_exp_minutes, "refresh_token".to_string()).map_err(|_e| {
            custom_error_response("Failed to create token.", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        // cookieヘッダーの生成
        let access_token_cookie = format!("access_token={}; HttpOnly; SameSite=Strict; max-age={}; Path=/", access_token, CONFIG.access_token_exp_minutes * 60);
        let refresh_token_cookie = format!("refresh_token={}; HttpOnly; SameSite=Strict; max-age={}; Path=/account/refresh", refresh_token, CONFIG.refresh_token_exp_minutes * 60);


        let access_token_cookie_header = HeaderValue::from_str(&access_token_cookie).map_err(|_e| {
            custom_error_response("Failed to set cookie.", StatusCode::INTERNAL_SERVER_ERROR)})?;
        let refresh_token_cookie_header = HeaderValue::from_str(&refresh_token_cookie).map_err(|_e| {
            custom_error_response("Failed to set cookie.", StatusCode::INTERNAL_SERVER_ERROR)})?;

        let body = json!({
            "success": true,
            "user": payload.username,
            "id": user.id,
            "totp_required": false,
        }).to_string();

        let mut builder = Response::builder();
        if let Some(headers) = builder.headers_mut() {
            headers.append("Set-Cookie", access_token_cookie_header);
            headers.append("Set-Cookie", refresh_token_cookie_header);
        }
    
        // レスポンスの生成
        let response = builder
            .status(StatusCode::OK)
            .body(body)
            .map_err(|_e| {
                custom_error_response(
                    "Failed to create response body.",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        )?;
        Ok(response)
    }
}

// 認証確認ハンドラー
pub async fn auth_check_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<AuthenticatedUser>, impl IntoResponse> {

    // SQLクエリの実行
    let result = query_as!(
        AuthenticatedUser,
        "SELECT id, username FROM user_model WHERE id = $1",
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(user) => Ok(Json(user)),
        Err(e) => match e {
            SqlxError::RowNotFound => Err(custom_error_response(
                "User not found.",
                StatusCode::NOT_FOUND,
            )),
            _ => Err(custom_error_response(
                "Database error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
    }
}

fn parsed_i64_to_string(string_int: String) -> Result<i64, std::num::ParseIntError> {
    match string_int.parse::<i64>() {
        Ok(parsed_int) => return Ok(parsed_int),
        Err(e) => return Err(e)
    };
}

// リフレッシュトークンの再取得ハンドラ
pub async fn refresh_token_handler(
    Extension(user_id): Extension<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {

    match refresh_access_token(user_id) {
        Ok(new_tokens) => {
            // cookieヘッダーの生成
            let access_token_cookie = format!("access_token={}; HttpOnly; SameSite=Strict; max-age={}; Path=/", new_tokens.access_token, CONFIG.access_token_exp_minutes * 60);
            let access_token_cookie_header = HeaderValue::from_str(&access_token_cookie).map_err(|_e| {
                custom_error_response("Failed to set cookie.", StatusCode::INTERNAL_SERVER_ERROR)
            })?;
            
            let refresh_token_cookie = format!("refresh_token={}; HttpOnly; SameSite=Strict; max-age={}; Path=/account/refresh", new_tokens.refresh_token, CONFIG.refresh_token_exp_minutes * 60);
            let refresh_token_cookie_header = HeaderValue::from_str(&refresh_token_cookie).map_err(|_e| {
                custom_error_response("Failed to set cookie.", StatusCode::INTERNAL_SERVER_ERROR)
            })?;
            
            let mut builder = Response::builder();
            if let Some(headers) = builder.headers_mut() {
                headers.append("Set-Cookie", access_token_cookie_header);
                headers.append("Set-Cookie", refresh_token_cookie_header);
            }
            
            // レスポンスの生成
            let response = builder
                .status(StatusCode::OK)
                .body(axum::body::Body::empty())
                .map_err(|_e| {
                    custom_error_response(
                        "Failed to create response body.",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            )?;
            Ok(response)
        }
        Err(err) => {
            tracing::error!("{}", err);
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
}

// アカウントの非公開・非公開設定ハンドラー
pub async fn account_privacy_update_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<UpdateAccountPrivacyPayload>,
) -> Result<Json<MessageApi>, impl IntoResponse> {
    let result = query!(
        "UPDATE user_model SET is_private = $1 WHERE id = $2",
        payload.is_private,
        user_id,
    )
    .execute(&pool)
    .await;

    match result {
        Ok(query_result) => {
            let affected_rows = query_result.rows_affected();
            if affected_rows > 0 {
                Ok(Json(MessageApi { message: "User privacy successfully updated.".to_string() }))
            } else {
                Err(custom_error_response(
                    "Not update user.",
                    StatusCode::BAD_REQUEST,
                ))
            }
        },
        Err(_) => Err(custom_error_response(
            "User privacy update failed.",
            StatusCode::BAD_REQUEST,
        ))
    }
}

// アカウント情報取得ハンドラー
pub async fn get_account_info_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<AccountPrivacyInfo>, impl IntoResponse> {
    let result = query_as!(
        AccountPrivacyInfo,
        "SELECT is_private, totp_secret FROM user_model WHERE id = $1",
        user_id,
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(query_result) => {
            Ok(Json(AccountPrivacyInfo { 
                is_private: query_result.is_private,
                totp_secret: query_result.totp_secret,
            }))
        },
        Err(_) => Err(custom_error_response(
            "User info get failed.",
            StatusCode::BAD_REQUEST,
        ))
    }
}

fn parse_naive_datetime(s: &str) -> Option<NaiveDateTime> {
    // 小数秒あり/なし、スペース/T 区切りの両方を許容
    let fmts: [&str; 4] = [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
    ];
    for f in fmts {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, f) {
            return Some(dt);
        }
    }
    None
}

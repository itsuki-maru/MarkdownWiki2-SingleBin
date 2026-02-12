use crate::config::CONFIG;
use crate::error::AppError;
use axum::{
    body::Body,
    extract::{Extension, Path},
    http::{HeaderValue, Response, StatusCode, header::CONTENT_TYPE},
    response::Response as HttpResponse,
};
use rust_embed::RustEmbed;
use sqlx::query_as;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(RustEmbed)]
#[folder = "dist/assets"]
struct Asset;

// 静的ファイルのレスポンスハンドラー
pub async fn serve_static_file(Path(uri): Path<String>) -> Result<Response<Body>, AppError> {
    match Asset::get(&uri) {
        Some(content) => {
            // 指定されたファイル名を検証する（ディレクトリトラバーサル攻撃対策）
            if let Some(safe_file_name) = sanitoze_filename(&uri) {
                let content_type = match safe_file_name.rsplit('.').next() {
                    Some("css") => "text/css",
                    Some("js") | Some("mjs") => "application/javascript",
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("html") => "text/html",
                    // 他の拡張子があれば適宜追加
                    _ => "application/octet-stream", // 不明なファイルタイプ
                };

                let body = content.data.into_owned();
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, content_type)
                    .body(body.into())
                    .expect("Failed to construct response");
                Ok(response)
            } else {
                Err(AppError::NotFound)
            }
        },
        None => Err(AppError::NotFound),
    }
}

// USER UPLOAD IMAGE RESPONSE
pub async fn serve_image_file(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(image_name): Path<String>,
) -> Result<Response<Body>, AppError> {
    // 指定されたファイル名を検証する（ディレクトリトラバーサル攻撃対策）
    if let Some(safe_file_name) = sanitoze_filename(&image_name) {
        struct ImageOwner {
            user_id: String,
        }

        struct IsPrivateUser {
            is_private: bool,
        }

        // 非公開ユーザーの画像データでないか検証
        let query_filen_name = image_name.clone();
        let owner = query_as!(
            ImageOwner,
            r#"
            SELECT user_id
            FROM image_model
            WHERE uuid_filename = $1
            "#,
            query_filen_name,
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to database access");
            AppError::Sqlx(e)
        })?;

        let owner = match owner {
            Some(owner) => owner,
            None => return Err(AppError::BadRequest),
        };

        // 画像のオーナーとアクセスしたユーザーのIDが異なる場合
        if owner.user_id != user_id {
            let is_private_user_db = query_as!(
                IsPrivateUser,
                r#"
                SELECT is_private
                FROM user_model
                WHERE id = $1
                "#,
                owner.user_id,
            )
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to database access");
                AppError::Sqlx(e)
            })?;

            // プライバシー設定がされている場合は NOT FOUND
            if is_private_user_db.is_private {
                return Err(AppError::NotFound);
            }
        }

        // ファイル名のUUID文字列から先頭5文字を取得
        let sub_dir = &safe_file_name[0..5];

        // パスを結合
        let mut base_path = PathBuf::from(&CONFIG.images_path);
        base_path.push(sub_dir);
        base_path.push(safe_file_name.trim_start_matches('/'));
        if !base_path.exists() || base_path.is_dir() {
            return Err(AppError::NotFound);
        }

        let content_type = match safe_file_name.rsplit('.').next() {
            Some("jpg") | Some("JPG") | Some("jpeg") => "image/jpeg",
            Some("png") | Some("PNG") => "image/png",
            Some("gif") | Some("GIF") => "image/gif",
            Some("webp") | Some("WEBP") => "image/webp",
            Some("mp4") | Some("MP4") => "video/mp4",
            Some("pdf") => "application/pdf",
            _ => "application/octet-stream", // 不明なファイルタイプ
        };

        // parseに失敗した場合はINTERNAL_SERVER_ERRORを早期リターン
        let parsed_content_type = content_type
            .parse()
            .map_err(|_e| AppError::InternalServerError)?;

        let mut builder = HttpResponse::builder();
        if let Some(headers) = builder.headers_mut() {
            headers.append(
                "Cache-Control",
                HeaderValue::from_static(&CONFIG.cache_control),
            );
            headers.append(CONTENT_TYPE, parsed_content_type);
        }

        let file = match File::open(base_path).await {
            Ok(file) => file,
            Err(_) => return Err(AppError::NotFound),
        };

        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::from_stream(stream);

        let response = builder
            .status(StatusCode::OK)
            .body(body)
            .expect("Failed to construct response");

        Ok(response)

    // 存在しないファイル
    } else {
        Err(AppError::NotFound)
    }
}

// SANITAIZE UPLOAD IMAGE FILENAME
fn sanitoze_filename(file_name: &str) -> Option<String> {
    let file_name = file_name.split('/').last()?;

    if file_name.contains("..") || file_name.contains("\\") || file_name.contains("/") {
        return None;
    }
    Some(file_name.to_string())
}

use axum::{
    body::Body, extract::{Extension, Path},
    http::{header::CONTENT_TYPE, HeaderValue, Response, StatusCode},
    response::{IntoResponse, Response as HttpResponse},
};
use sqlx::sqlite::SqlitePool;
use sqlx::query_as;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use std::path::PathBuf;
use crate::config::CONFIG;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "dist/assets"]
struct Asset;

// 静的ファイルのレスポンスハンドラー
pub async fn serve_static_file(
    Path(uri): Path<String>,
) -> Result<Response<Body>, impl IntoResponse> {
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
                Err(StatusCode::NOT_FOUND)
            }
        }
        None => Err(StatusCode::NOT_FOUND)
    }
}

// USER UPLOAD IMAGE RESPONSE
pub async fn serve_image_file(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(image_name): Path<String>,
) -> Result<Response<Body>, impl IntoResponse> {

    // 指定されたファイル名を検証する（ディレクトリトラバーサル攻撃対策）
    if let Some(safe_file_name) = sanitoze_filename(&image_name) {

        struct ImageOwner {
            user_id: String,
        }

        struct IsPrivateUser {
            is_private: bool,
        }

        // 非公開ユーザーの画像データでないか検証
        let result = query_as!(
            ImageOwner,
            "SELECT user_id FROM image_model WHERE uuid_filename = $1",
            image_name,
        )
        .fetch_one(&pool)
        .await;

        match result {
            Ok(owner) => {
                // 画像のオーナーとアクセスしたユーザーのIDが異なる場合
                if owner.user_id != user_id {
                    let result = query_as!(
                        IsPrivateUser,
                        "SELECT is_private FROM user_model WHERE id = $1",
                        owner.user_id,
                    )
                    .fetch_one(&pool)
                    .await;
    
                    match result {
                        Ok(user) => {
                            if user.is_private {
                                return Err(StatusCode::NOT_FOUND);
                            }
                        },
                        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            },
            Err(_err) => {
                return Err(StatusCode::NOT_FOUND)
            }
        }


        // ファイル名のUUID文字列から先頭5文字を取得
        let sub_dir = &safe_file_name[0..5];

        // パスを結合
        let mut base_path = PathBuf::from(&CONFIG.images_path);
        base_path.push(sub_dir);
        base_path.push(safe_file_name.trim_start_matches('/'));
        if !base_path.exists() || base_path.is_dir() {
            return Err(StatusCode::NOT_FOUND);
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
            .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut builder = HttpResponse::builder();
        if let Some(headers) = builder.headers_mut() {
            headers.append("Cache-Control", HeaderValue::from_static(&CONFIG.cache_control));
            headers.append(CONTENT_TYPE, parsed_content_type);
        }

        let file = match File::open(&base_path).await {
            Ok(file) => file,
            Err(_) => return Err(StatusCode::NOT_FOUND),
        };

        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::from_stream(stream);

        let response = builder
            .status(StatusCode::OK)
            .body(body)
            .expect("Failed to construct response");

        Ok(response)
    } else {
        Err(StatusCode::NOT_FOUND)
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

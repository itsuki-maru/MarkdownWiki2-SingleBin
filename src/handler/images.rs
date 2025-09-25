use axum::{
    extract::{Path, Extension},
    response::IntoResponse,
    http::StatusCode, Json,
};
use futures_util::TryStreamExt as _;
use sqlx::sqlite::SqlitePool;
use sqlx::query_as;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::io::StreamReader;
use uuid::Uuid;
use image::{ImageFormat, io::Reader as ImageReader};
use std::io::Cursor;
use std::collections::HashMap;
use std::path::Path as StdPath;
use std::path::PathBuf;
use chrono::Utc;
use super::super::custom_responses::custom_error_response;
use super::super::scheme::{
    DeletedImageResponse,
    ImageData,
    ReturningId,
    ImageIdNameDeleted,
    UploadResponseImage,
};
use super::super::image_ext_validator::check_file_extension;
use super::super::utils::ensure_dir;
use crate::config::CONFIG;

// GET IMAGES LIMIT
pub async fn get_enable_images_limit_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(limit): Path<i64>,
) -> Result<Json<HashMap<String, ImageData>>, impl IntoResponse> {

    let result = query_as!(
        ImageData,
        "SELECT id, user_id, filename, uuid_filename FROM image_model WHERE user_id = $1 ORDER BY id DESC LIMIT $2",
        user_id,
        limit,
    )
    .fetch_all(&pool)
    .await;

    let images = match result {
        Ok(images) => images,
        Err(_) => {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let mut images_hash_map = HashMap::new();
    for image in images {
        let image_id = image.id.clone();
        images_hash_map.insert(image_id, image);
    }

    Ok(Json(images_hash_map))
}

// GET ALL IMAGES
pub async fn get_enable_images_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<HashMap<String, ImageData>>, impl IntoResponse> {

    let result = query_as!(
        ImageData,
        "SELECT id, user_id, filename, uuid_filename FROM image_model WHERE user_id = $1",
        user_id,
    )
    .fetch_all(&pool)
    .await;

    let images = match result {
        Ok(images) => images,
        Err(_) => {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let mut images_hash_map = HashMap::new();
    for image in images {
        let image_id = image.id.clone();
        images_hash_map.insert(image_id, image);
    }

    Ok(Json(images_hash_map))
}

// UPLOAD IMAGE
pub async fn upload_image_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    mut payload: axum::extract::Multipart,
) -> Result<Json<UploadResponseImage>, impl IntoResponse> {

    // 現在時刻を取得
    let now = Utc::now().naive_utc();

    // 新規ID
    let new_image_id = Uuid::now_v7().to_string();
    let mut original_filename = String::new();
    let mut unique_filename = String::new();
    while let Some(field) = payload
        .next_field()
        .await
        .map_err(|_e| StatusCode::BAD_REQUEST)?
    {
        // UUIDを生成
        let uuid = Uuid::now_v7();

        // UUID文字列から先頭5文字を取得
        let sub_dir = &uuid.to_string()[0..5];

        // 保存先とするディレクトリパスを作成
        let dir_path = PathBuf::from(CONFIG.upload_file_path.clone()).join(sub_dir);

        match ensure_dir(&dir_path).await {
            Ok(_) => {},
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }

        // アップロードされたファイル名を取得
        let original_name = field.file_name().unwrap_or("file").to_string();
        // Content-Typeを取得
        let content_type = field.content_type().unwrap_or("image/").to_string();

        let file_name_path = StdPath::new(&original_name);
        let ext = match file_name_path.extension() {
            Some(ext) => ext.to_string_lossy(),
            None => return Err(StatusCode::BAD_REQUEST),
        };

        // 拡張子によるファイル検査
        let valid_ext = match check_file_extension(ext.to_string()) {
            Ok(valid_ext) => valid_ext,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        // 画像ファイルの場合EXIF情報などを除去して保存
        if content_type.starts_with("image/") {
            // 一時ファイル名を設定
            let temp_file_path = format!("{}/temp_{}.{}", CONFIG.upload_file_path, uuid, valid_ext);
            // 一時ファイルを作成
            let mut temp_file = match File::create(&temp_file_path).await {
                Ok(file) => file,
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            };

            // 作成した一時ファイルにストリームでデータを流し込む
            let mut stream =
            StreamReader::new(field.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

            if let Err(_) = tokio::io::copy(&mut stream, &mut temp_file).await {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            // 画像を読み込みEXIF情報を削除し、元の形式で保存
            let temp_file_data = tokio::fs::read(&temp_file_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let img_reader = ImageReader::new(Cursor::new(&temp_file_data))
                .with_guessed_format()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let img = &img_reader.decode()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let format = match ImageFormat::from_path(&temp_file_path) {
                Ok(format) => format,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            };

            // 最終ファイルを作成
            let final_file_path = format!("{}/{}.{}", dir_path.to_string_lossy(), uuid, valid_ext);
            let mut final_file = File::create(&final_file_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let mut output_data = Vec::new();
            img.write_to(&mut Cursor::new(&mut output_data), format)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            final_file.write_all(&output_data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // 一時ファイルを削除
            tokio::fs::remove_file(&temp_file_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;


        // PDFファイルや動画ファイルの処理
        } else {
            // 保存ファイル名を設定
            let file_path = format!("{}/{}.{}", dir_path.to_string_lossy(), uuid, valid_ext);
            
            // ファイルを作成
            let mut file = match File::create(file_path).await {
                Ok(file) => file,
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            };

            // 作成したファイルにストリームでデータを流し込む
            let mut stream =
            StreamReader::new(field.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
            
            if let Err(_) = tokio::io::copy(&mut stream, &mut file).await {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }

        unique_filename = format!("{}.{}", uuid.to_string(), ext);

        let save_image_id = new_image_id.clone();
        let save_unique_filename = unique_filename.clone();

        // DBに保存する処理
        let rec = query_as!(
            ReturningId,
            "INSERT INTO image_model (id, user_id, filename, uuid_filename, create_at)
            VALUES ($1, $2, $3, $4, $5) RETURNING id",
            save_image_id,
            user_id,
            original_name,
            save_unique_filename,
            now,
        )
        .fetch_one(&pool)
        .await;

        match rec {
            Ok(_) => {
                original_filename = original_name;
            }
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    }
    Ok(Json(UploadResponseImage {
        new_image_id: new_image_id,
        user_id: user_id,
        filename: original_filename,
        uuid_filename: unique_filename,
    }))
}

// DELETE IMAGE
pub async fn delete_image_handler(
    Extension(user_id): Extension<String>,
    Extension(pool): Extension<SqlitePool>,
    Path(image_id): Path<String>,
) -> Result<Json<DeletedImageResponse>, impl IntoResponse> {

    // UUID文字列から先頭5文字を取得
    let sub_dir = &image_id.to_string()[0..5];

    // サブディレクトリパス
    let dir_path = PathBuf::from(CONFIG.upload_file_path.clone()).join(sub_dir);

    let result = query_as!(
        ImageIdNameDeleted,
        "DELETE FROM image_model WHERE id = $1 AND user_id = $2 RETURNING id, uuid_filename",
        image_id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(deleted_image) => {
            let file_path = format!("{}/{}", dir_path.to_string_lossy(), &deleted_image.uuid_filename);
            match std::fs::remove_file(file_path) {
                Ok(_) => {
                    return Ok(Json(DeletedImageResponse {
                        id: deleted_image.id,
                        message: "Delete Ok.".to_string(),
                    }))
                }
                Err(_) => {
                    return Err(custom_error_response(
                        "Internal Server Error.",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(_) => {
            return Err(custom_error_response(
                "Internal Server Error.",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
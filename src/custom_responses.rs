use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;
use std::convert::Infallible;

// エラーレスポンス用のヘルパー関数
pub fn custom_error_response(
    message: &str,
    status: StatusCode,
) -> Result<impl IntoResponse, Infallible> {
    let body = json!({ "error": message }).to_string();
    let response = Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|_e| {});
    // Infallibleを使用しているため、ここでのErrは発生しえない（Infallibleはエラーが決して起こらないことを表す型）
    Ok(response)
}

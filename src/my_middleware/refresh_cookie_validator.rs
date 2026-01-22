use crate::auth::verify_refresh_token;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use serde_json::json;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RefreshCookieValidator;

impl<S> Layer<S> for RefreshCookieValidator {
    type Service = RefreshCookieValidatorMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RefreshCookieValidatorMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct RefreshCookieValidatorMiddleware<S> {
    inner: S,
}

impl <S, B> Service<Request<B>> for RefreshCookieValidatorMiddleware<S>
    where
        S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
        S::Future: Send + 'static,
        B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    // リクエストを受け取り、処理を行い、結果を返すメソッド。ここでミドルウェアの主な処理が行う
    fn call(&mut self, req: Request<B>) -> Self::Future {
        let mut inner = self.inner.clone();

        let future = async move {
            // リクエストヘッダーからCookieを取得
            let cookies_str = req
                .headers()
                .get(axum::http::header::COOKIE)
                .and_then(|header_value| header_value.to_str().ok())
                .unwrap_or("");

            // Cookieヘッダーをセミコロンで分割して各cookieを解析
            let cookies: Vec<&str> = cookies_str.split(';').collect();

            // refresh_tokenをCookieヘッダーから取り出す
            let mut refresh_token_value = None;
            for cookie in cookies {
                let parts: Vec<&str> = cookie.split('=').map(|part| part.trim()).collect();
                if parts.len() == 2 && parts[0] == "refresh_token" {
                    refresh_token_value = Some(parts[1]);
                    break;
                }
            }

            // リフレッシュトークンを検証し結果に応じてレスポンス
            match verify_refresh_token(&refresh_token_value.unwrap_or("")) {
                // 成功時はハンドラーからユーザーIDを取り出せるよう設定
                Ok(claims) => {
                    // ユーザーIDをリクエストのExtensionとしてセット
                    let mut req = req;
                    req.extensions_mut().insert(claims.sub);
                    // 内部サービスへのリクエストを続ける
                    return inner.call(req).await;
                }
                // 検証失敗時のレスポンス
                Err(_) => {
                    let response_body = json!({
                        "error": "refresh_token_expired"
                    });
                    let response = axum::response::Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header("Content-Type", "application/json")
                        .body(axum::body::Body::from(response_body.to_string()));
                    match response {
                        Ok(response) => return Ok(response),
                        Err(err) => {
                            tracing::error!("{}", err);
                            let response = (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response();
                            return Ok(response);
                        }
                    }
                }
            }
        };
        Box::pin(future)
    }
}

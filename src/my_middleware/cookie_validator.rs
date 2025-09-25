use super::super::auth::verify_access_token;
use axum::response::IntoResponse;
use serde_json::json;
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service}; // ミドルウェアを作成するためのトレイト

// CookieValidatorはクローン可能なマーカー構造体で、ミドルウェアのレイヤーを提供
#[derive(Clone)]
pub struct CookieValidator;

// Layerトレイトを実装して、CookieValidatorMiddlewareをミドルウェアとして機能させる
impl<S> Layer<S> for CookieValidator {
    type Service = CookieValidatorMiddleware<S>;

    // レイヤーを適用するためのメソッド。ここで実際のミドルウェアのインスタンスを作成
    fn layer(&self, inner: S) -> Self::Service {
        CookieValidatorMiddleware { inner }
    }
}

// ミドルウェアの実装部分。具体的な処理はこの構造体内で行われる
#[derive(Clone)]
pub struct CookieValidatorMiddleware<S> {
    inner: S,
}

// Serviceトレイトを実装して、リクエストを受け取り、処理を行い、レスポンスを返す機能を定義
impl <S, B> Service<Request<B>> for CookieValidatorMiddleware<S>
    where
        S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static, // 制約を定義
        S::Future: Send + 'static, // 非同期処理のためのFuture型がSendトレイトを実装している必要がある
        B: Send + 'static, // リクエストボディもSendトレイトを実装している必要がある
{
    type Response = S::Response; // SからResponse型を継承（レスポンス型を内部サービスのものに設定）
    type Error = S::Error; // エラー型も内部サービスから継承
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>; // 非同期処理の型を定義

    // サービスが次のリクエストを受け取る準備ができているかをポーリングするメソッド
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

            // access_tokenをCookieヘッダーから取り出す
            let mut access_token_value = None;
            for cookie in cookies {
                let parts: Vec<&str> = cookie.split('=').map(|part| part.trim()).collect();
                if parts.len() == 2 && parts[0] == "access_token" {
                    access_token_value = Some(parts[1]);
                    break;
                }
            }

            // アクセストークンを検証し結果に応じてレスポンス
            match verify_access_token(&access_token_value.unwrap_or("")) {
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
                        "error": "token_expired"
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
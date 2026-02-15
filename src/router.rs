use axum::{
    Router,
    extract::{DefaultBodyLimit, Extension},
    http::{
        Method,
        header::{self, HeaderName, HeaderValue},
    },
    middleware,
    routing::{delete, get, post, put},
};
use sqlx::sqlite::SqlitePool;
use std::str::FromStr;
use std::sync::Arc;
use tera::Tera;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::config::CONFIG;
use crate::handler::account::{
    account_privacy_update_handler, auth_check_handler, disable_token, get_account_info_handler,
    refresh_token_handler, signup_handler, token_handler,
};
use crate::handler::admin::{
    admin_index_get_handler, create_users_handler, get_users_handler, unlock_account_handler,
    update_public_name_handler, update_users_password_handler,
};
use crate::handler::assets::{serve_image_file, serve_static_file};
use crate::handler::images::{
    delete_image_handler, get_enable_images_handler, get_enable_images_limit_handler,
    upload_image_handler,
};
use crate::handler::onetime_url::{
    generate_url_handler, get_all_temporary_urls, invalidate_url_handler,
    temporary_wiki_get_handler,
};
use crate::handler::totp::{
    token_totp_handler, totp_disable_handler, totp_setup_handler, totp_verify_handler,
};
use crate::handler::wiki::{
    create_wiki_handler, delete_wiki_handler, download_file, get_all_wiki_handler,
    get_wiki_by_id_handler, get_wiki_limit_handler, get_wiki_owner_handler, update_wiki_handler,
    wiki_query_handler,
};
use crate::handler::wiki_edit::{
    disable_edit_request, edit_request_owner_result, get_edit_request_wikis, request_wiki_edit,
};
use crate::middleware::{
    cookie_validator::CookieValidator, flexible_cookie_validator::FlexibleCookieValidator,
    print_req_res::print_request_response, refresh_cookie_validator::RefreshCookieValidator,
};

pub fn build_router(pool: SqlitePool, tera: Arc<Mutex<Tera>>) -> Router {
    // CORSの設定
    let mut cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            HeaderName::from_str("X-Requested-With").unwrap(),
        ])
        .allow_credentials(true)
        .expose_headers(vec![header::CONTENT_TYPE]);

    // 開発時のみ Vue3 のサーバを許可オリジンに追加
    if cfg!(debug_assertions) {
        cors = cors.allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap());
    }

    // アクセストークンによる認可を要する
    let secured_routes = Router::new()
        .route("/wiki/add", post(create_wiki_handler))
        .route("/wiki/read/all", get(get_all_wiki_handler))
        .route("/wiki/read/all/count/{limit}", get(get_wiki_limit_handler))
        .route("/wiki/read/{wiki_id}", get(get_wiki_by_id_handler))
        .route("/wiki/owner/{wiki_id}", get(get_wiki_owner_handler))
        .route("/wiki/modify/{wiki_id}", put(update_wiki_handler))
        .route("/wiki/remove/{wiki_id}", delete(delete_wiki_handler))
        .route("/wiki/query", get(wiki_query_handler))
        .route("/wiki/download/{wiki_id}", get(download_file))
        .route("/images/eneble-images", get(get_enable_images_handler))
        .route(
            "/images/eneble-images/{limit}",
            get(get_enable_images_limit_handler),
        )
        .route("/images/upload", post(upload_image_handler))
        .route("/images/delete/{image_id}", delete(delete_image_handler))
        .route("/account/auth", get(auth_check_handler))
        .route("/admin", get(admin_index_get_handler))
        .route("/admin/users", get(get_users_handler))
        .route(
            "/admin/user/password-reset/{update_user_id}",
            post(update_users_password_handler),
        )
        .route(
            "/admin/user/publicname-update/{update_user_id}",
            put(update_public_name_handler),
        )
        .route(
            "/admin/user/unlock/{unlock_user_id}",
            post(unlock_account_handler),
        )
        .route("/admin/user/create", post(create_users_handler))
        .route("/onetimeurl/generate/{wiki_id}", post(generate_url_handler))
        .route(
            "/onetimeurl/delete/{id_url}",
            delete(invalidate_url_handler),
        )
        .route("/onetimeurl/all", get(get_all_temporary_urls))
        .route("/account/info", get(get_account_info_handler))
        .route("/account/privacy", put(account_privacy_update_handler))
        .route("/account/totp/setup", get(totp_setup_handler))
        .route("/account/totp/verify", post(totp_verify_handler))
        .route("/account/totp/disable", get(totp_disable_handler))
        .route("/account/token/disable", get(disable_token))
        .route("/wiki-edit/request/{wiki_id}", put(request_wiki_edit))
        .route("/wiki-edit/lists", get(get_edit_request_wikis))
        .route("/wiki-edit/result", post(edit_request_owner_result))
        .route(
            "/wiki-edit/disable/{edit_request_wiki_id}",
            delete(disable_edit_request),
        )
        .layer(CookieValidator);

    // アクセストークン不要
    let mut not_secure_routes = Router::new()
        .route("/", get(crate::root_handler))
        .route("/index", get(crate::index_handler))
        .route("/health-check", get(crate::health_check_handler))
        .route("/app-init", get(crate::get_app_init_handler))
        .route("/favicon.ico", get(crate::serve_favicon))
        .route("/assets/{uri}", get(serve_static_file))
        .route("/account/token", post(token_handler))
        .route("/account/totp/token", post(token_totp_handler))
        .route("/onetime/{url_id}", get(temporary_wiki_get_handler))
        .route("/licanses", get(crate::licenses_get_handler));

    if CONFIG.allow_user_create_account {
        not_secure_routes = not_secure_routes.route("/account/signup", post(signup_handler));
    }

    // リフレッシュトークンを要する
    let token_refresh_routes = Router::new()
        .route("/account/refresh", post(refresh_token_handler))
        .layer(RefreshCookieValidator);

    // アクセストークンを持たない場合においても内部サービスへ接続
    let flex_secured_routes = Router::new()
        .route("/static/images/{image_name}", get(serve_image_file))
        .layer(FlexibleCookieValidator);

    // 最終的なAPIルート
    Router::new()
        .merge(secured_routes)
        .merge(not_secure_routes)
        .merge(token_refresh_routes)
        .merge(flex_secured_routes)
        .layer(cors)
        .layer(Extension(pool)) // PostgreSQLへの接続は全てに適用
        .layer(middleware::from_fn(print_request_response))
        .layer(DefaultBodyLimit::max(30 * 1024 * 1024)) // ファイルサイズ上限を30MBに設定
        .layer(Extension(tera))
        .fallback(crate::custom_not_found_handler)
}

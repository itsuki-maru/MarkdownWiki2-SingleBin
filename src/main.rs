use axum::{
    body::Body,
    extract::{Extension, DefaultBodyLimit},
    http::{
        header::{self, HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE},
        Method, Request, Response, StatusCode},
    middleware, response::
    {Html, IntoResponse, Redirect},
    routing::{delete, get, post, put}, Json, Router
};
use webbrowser;
use rust_embed::RustEmbed;
use tera::Tera;
use clap::{Command, Arg};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;

mod auth;
mod custom_responses;
mod image_ext_validator;
mod database;
mod handler;
mod my_middleware;
mod scheme;
mod init;

use database::{
    setup_database_pool,
    check_and_insert_initial_data,
};
use handler::account::{
    auth_check_handler,
    signup_handler,
    token_handler,
    refresh_token_handler,
    account_privacy_update_handler,
    get_account_info_handler,
};
use handler::admin::{
    admin_index_get_handler,
    get_users_handler,
    unlock_account_handler,
    create_users_handler,
    update_users_password_handler,
    update_public_name_handler,
};
use handler::assets::{
    serve_image_file,
    serve_static_file,
};
use handler::images::{
    delete_image_handler,
    get_enable_images_handler,
    get_enable_images_limit_handler,
    upload_image_handler,
};
use handler::wiki::{
    create_wiki_handler,
    delete_wiki_handler,
    download_file,
    get_all_wiki_handler,
    get_wiki_by_id_handler,
    get_wiki_limit_handler,
    get_wiki_owner_handler,
    update_wiki_handler,
    wiki_query_handler,
};
use handler::onetime_url::{
    generate_url_handler,
    temporary_wiki_get_handler,
    invalidate_url_handler,
    get_all_temporary_urls,
};
use handler::totp::{
    token_totp_handler,
    totp_verify_handler,
    totp_setup_handler,
    totp_disable_handler,
};
use my_middleware::{
    cookie_validator::CookieValidator,
    print_req_res::print_request_response,
    refresh_cookie_validator::RefreshCookieValidator,
    flexible_cookie_validator::FlexibleCookieValidator,
};

use init::{
    read_or_create_json_env,
    get_application_user_setup_path,
};

use scheme::{MessageApi, AppInit};
mod config;
mod utils;
use config::CONFIG;

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Asset;

#[derive(RustEmbed)]
#[folder = "dist/templates/"]
struct Templates;

#[tokio::main]
async fn main() {
    // CLI定義
    let cli = Command::new("MarkdownWiki2")
        .version("1.1.6")
        .author("Itsuki Maru")
        .about("MarkdownWiki2 Backend Server")
        .disable_help_flag(true)
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .value_name("HostName")
                .required(false)
                .value_parser(clap::value_parser!(String))
                .default_value("localhost")
                .help("ex) -h localhost")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PortNumber")
                .required(false)
                .value_parser(clap::value_parser!(String))
                .default_value("3080")
                .help("ex) -p 3080")
        )
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .required(false)
                .help("ex) -s")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let mut host_ip_address: String = String::new();
    let mut host_port: String = String::new();
    let mut is_server_only = false;
    if let (Some(hostname), Some(port), is_server) = (
        cli.get_one::<String>("host"),
        cli.get_one::<String>("port"),
        cli.get_flag("server"),
    ) {
        host_ip_address = hostname.to_string();
        host_port = port.to_string();
        is_server_only = is_server;
    }

    // 起動ソケット
    let addr = format!("{}:{}", host_ip_address, host_port);

    // 初期化処理
    let app_setup_path = get_application_user_setup_path();
    let default_env = read_or_create_json_env(app_setup_path);

    // 環境変数設定
    // Rustの2024エディション以降、env::set_varはunsafe fnに変更されたため、unsafeを使用。アプリケーションの初期化に限る。
    unsafe {
        env::set_var("APP_TITLE", default_env.app_title);
        env::set_var("CREATEDATABASE_PATH", default_env.sqlite_database_path);
        env::set_var("DATABASE_URL", default_env.database_url);
        env::set_var("SECRET_KEY", default_env.secret_key);
        env::set_var("IMAGE_FILES_PATH", default_env.image_file_path);
        env::set_var("UPLOAD_FILE_PATH", default_env.upload_file_path);
        env::set_var("FAILED_ACCOUNT_LOCK", default_env.failed_account_lock);
        env::set_var("NEXT_CHALLENGE_MINUTES", default_env.next_challenge_minutes);
        env::set_var("CHALLENGE_LIMIT_TIME_FAILEDCOUNT", default_env.challenge_limit_time_failed_count);
        env::set_var("ADMIN_USERNAME", default_env.admin_username);
        env::set_var("ADMIN_PASSWORD", default_env.admin_passwotd);
        env::set_var("ACCESS_TOKEN_EXP_MINUTUES", default_env.access_token_exp_minutes);
        env::set_var("REFRESH_TOKEN_EXP_MINUTUES", default_env.refresh_token_exp_minutes);
        env::set_var("CACHE_CONTROL", default_env.cache_control);
        env::set_var("SECURE_COOKIE", default_env.secure_cookie);
        env::set_var("SERVICE_NAME", default_env.service_name);
        env::set_var("RUST_LOG", default_env.rust_log);
        env::set_var("ALLOW_USER_CREATE_ACCOUNT", default_env.allow_user_create_account);
        env::set_var("ALLOW_ORIGINS", format!("{},http://{}", default_env.allow_origins, &addr));
    }

    // ログ設定
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_e| "Middleware Debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("==================== Server Startup ====================");

    // データベース接続コネクションを取得
    let pool = match setup_database_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to create pool: {}", e);
            return;
        }
    };

    // 初期データの存在を確認し、存在しなければ作成
    check_and_insert_initial_data(&pool).await.unwrap();

    // Teraの設定
    let tera = build_tera_from_embed().unwrap();
    let tera = Arc::new(Mutex::new(tera));

    // CORSの設定例
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
        .expose_headers(vec![
            header::CONTENT_TYPE,
            // 必要に応じて他のヘッダーを追加
        ]);

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
        .route("/images/eneble-images/{limit}", get(get_enable_images_limit_handler))
        .route("/images/upload", post(upload_image_handler))
        .route("/images/delete/{image_id}", delete(delete_image_handler))
        .route("/account/auth", get(auth_check_handler))
        .route("/admin", get(admin_index_get_handler))
        .route("/admin/users", get(get_users_handler))
        .route("/admin/user/password-reset/{update_user_id}", post(update_users_password_handler))
        .route("/admin/user/publicname-update/{update_user_id}", put(update_public_name_handler))
        .route("/admin/user/unlock/{unlock_user_id}", post(unlock_account_handler))
        .route("/admin/user/create", post(create_users_handler))
        .route("/onetimeurl/generate/{wiki_id}", post(generate_url_handler))
        .route("/onetimeurl/delete/{id_url}", delete(invalidate_url_handler))
        .route("/onetimeurl/all", get(get_all_temporary_urls))
        .route("/account/info", get(get_account_info_handler))
        .route("/account/privacy", put(account_privacy_update_handler))
        .route("/account/totp/setup", get(totp_setup_handler))
        .route("/account/totp/verify", post(totp_verify_handler))
        .route("/account/totp/disable", get(totp_disable_handler))
        .layer(CookieValidator);

    // アクセストークン不要
    let mut not_secure_routes = Router::new()
        .route("/", get(root_handler))
        .route("/index", get(index_handler))
        .route("/health-check", get(health_check_handler))
        .route("/app-init", get(get_app_init_handler))
        .route("/favicon.ico", get(serve_favicon))
        .route("/assets/{uri}", get(serve_static_file))
        .route("/account/token", post(token_handler))
        .route("/account/totp/token", post(token_totp_handler))
        .route("/onetime/{url_id}", get(temporary_wiki_get_handler))
        .route("/licanses", get(licenses_get_handler));

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

    // 最終的なルーター
    let app = Router::new()
        .merge(secured_routes)
        .merge(not_secure_routes)
        .merge(token_refresh_routes)
        .merge(flex_secured_routes)
        .layer(cors)
        .layer(Extension(pool)) // PostgreSQLへの接続は全てに適用
        .layer(middleware::from_fn(print_request_response))
        .layer(DefaultBodyLimit::max(30 * 1024 * 1024)) // ファイルサイズ上限を30MBに設定
        .layer(Extension(tera))
        .fallback(custom_not_found_handler);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    // サーバーモードでなければブラウザ起動（-sオプションなしの場合）
    if !is_server_only {
        let url = format!("http://{}", &addr);
        if webbrowser::open(&url).is_ok() {
            tracing::info!("=================== Open Web Browser ===================");
        }
    }

    tracing::info!("========== Listening on http://{} ==========", addr);
    axum::serve(listener, app).await.unwrap();
}

// ルートへのアクセスは /index にリダイレクト
async fn root_handler() -> impl IntoResponse {
    Redirect::permanent("/index")
}

// アプリケーション初期設定情報の取得ハンドラ
async fn get_app_init_handler(_: Request<Body>) -> Json<AppInit> {
    Json(
        AppInit { 
            app_title: CONFIG.app_title.clone(),
            allow_user_account_create: CONFIG.allow_user_create_account,
            allow_origins: CONFIG.allow_origins.clone(),
        }
    )
}

// 死活監視用API
async fn health_check_handler() -> Json<MessageApi> {
    Json(MessageApi {
        message: "Hello, I'm administrator.".to_string(),
    })
}

// INDEX HTML GET HANDLER
async fn index_handler(
    headers: HeaderMap,
) -> Result<Html<String>, impl IntoResponse> {
    // User-Agent取り出し
    let user_agent = headers.get("user-agent").and_then(|ua| ua.to_str().ok());

    // User-Agentに"Mobile"が含まれているか確認
    let is_mobile = user_agent.map_or(false, |ua| ua.contains("Mobile"));

    tracing::debug!("User-Agent: {:?}", user_agent);
    tracing::debug!("Is Mobile: {}", is_mobile);

    let render_html = if is_mobile {
        "index-mobile.html"
    } else {
        "index.html"
    };

    match Asset::get(render_html) {
        Some(content) => {
            let html_content = String::from_utf8(content.data.into_owned()).unwrap();
            Ok(Html(html_content))
        }
        None => Err((StatusCode::NOT_FOUND, "Content not foubnd").into_response())
    }
}

async fn licenses_get_handler() -> Result<Html<String>, impl IntoResponse> {
    match Asset::get("licenses.html") {
        Some(content) => {
            let html_content = String::from_utf8(content.data.into_owned()).unwrap();
            Ok(Html(html_content))
        }
        None => Err((StatusCode::NOT_FOUND, "Content not foubnd").into_response())
    }
}

// 404 HANDLER
async fn custom_not_found_handler(_: Request<Body>) -> impl IntoResponse {
    Redirect::permanent("/index")
}

async fn serve_favicon() -> Result<Response<Body>, impl IntoResponse> {
    match Asset::get("assets/favicon.ico") {
        Some(content) => {
            let body = content.data.into_owned();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "image/x-icon")
                .body(body.into())
                .expect("Failed to construct response");
            Ok(response)
        }
        None => Err((StatusCode::NOT_FOUND, "Content not foubnd").into_response())
    }
}

// Teraにテンプレートファイルを rust_embed から登録する処理
fn build_tera_from_embed() -> anyhow::Result<Tera> {
    let mut tera = Tera::default();

    // RustEmbedに入っている全テンプレートを登録
    for path in Templates::iter() {
        let path_str = path.as_ref();
        if let Some(file) = Templates::get(path_str) {
            let content = std::str::from_utf8(file.data.as_ref())?; // UTF-8前提
            tera.add_raw_template(path_str, content)?;
        }
    }
    Ok(tera)
}
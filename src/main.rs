// Windowsでコンソールを非表示にする設定処理
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    Json,
    body::Body,
    http::{HeaderMap, Request, Response, StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse, Redirect},
};
use clap::{Arg, Command};
use rust_embed::RustEmbed;
use std::env;
use std::process;
use std::sync::Arc;
use tera::Tera;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use webbrowser;

mod auth;
mod config;
mod db;
mod error;
mod handler;
mod image;
mod init;
mod middleware;
mod model;
mod router;
mod utils;

use db::{check_and_insert_initial_data, setup_database_pool};

use init::{get_application_user_setup_path, read_or_create_json_env};
use model::{AppInit, MessageApi};

use config::CONFIG;
use error::AppError;
#[cfg(windows)]
use utils::ensure_console;

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
        .version("1.4.2")
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
                .default_value("127.0.0.1")
                .help("ex) -h 127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PortNumber")
                .required(false)
                .value_parser(clap::value_parser!(String))
                .default_value("3080")
                .help("ex) -p 3080"),
        )
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .required(false)
                .help("ex) -s")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("console")
                .short('c')
                .long("console")
                .required(false)
                .help("ex) -c Windows Only")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let mut host_ip_address: String = String::new();
    let mut host_port: String = String::new();
    let mut is_server_only = false;
    let mut is_show_console = false;
    if let (Some(hostname), Some(port), is_server, is_console) = (
        cli.get_one::<String>("host"),
        cli.get_one::<String>("port"),
        cli.get_flag("server"),
        cli.get_flag("console"),
    ) {
        host_ip_address = hostname.to_string();
        host_port = port.to_string();
        is_server_only = is_server;
        is_show_console = is_console;
    }

    // 起動ソケット
    let addr = format!("{}:{}", host_ip_address, host_port);
    let mut browser_url: String = match host_ip_address.trim() {
        // String から &str
        "127.0.0.1" => format!("http://localhost:{}", host_port),
        _ => format!("http://{}:{}", host_ip_address, host_port),
    };

    // すでにサーバが起動していないかOS問い合わせ
    if std::net::TcpListener::bind(&addr).is_err() {
        // バインドを試みエラーの場合は既に起動済みのためブラウザを開き終了
        tracing::info!("================ Server Already Started ================");
        if !is_server_only {
            if webbrowser::open(&browser_url).is_ok() {
                tracing::info!("=================== Open Web Browser ===================");
            }
        }
        process::exit(0);
    }

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
        env::set_var(
            "CHALLENGE_LIMIT_TIME_FAILEDCOUNT",
            default_env.challenge_limit_time_failed_count,
        );
        env::set_var("ADMIN_USERNAME", default_env.admin_username);
        env::set_var("ADMIN_PASSWORD", default_env.admin_passwotd);
        env::set_var(
            "ACCESS_TOKEN_EXP_MINUTUES",
            default_env.access_token_exp_minutes,
        );
        env::set_var(
            "REFRESH_TOKEN_EXP_MINUTUES",
            default_env.refresh_token_exp_minutes,
        );
        env::set_var("CACHE_CONTROL", default_env.cache_control);
        env::set_var("SECURE_COOKIE", default_env.secure_cookie);
        env::set_var("SERVICE_NAME", default_env.service_name);
        env::set_var("RUST_LOG", default_env.rust_log);
        env::set_var(
            "ALLOW_USER_CREATE_ACCOUNT",
            default_env.allow_user_create_account,
        );
        env::set_var(
            "ALLOW_ORIGINS",
            format!("{},http://{}", default_env.allow_origins, &addr),
        );
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
        },
    };

    // 初期データの存在を確認し、存在しなければ作成
    check_and_insert_initial_data(&pool).await.unwrap();

    // Teraの設定
    let tera = build_tera_from_embed().unwrap();
    let tera = Arc::new(Mutex::new(tera));

    // ルーター構築
    let app = router::build_router(pool, tera);

    // 開発時のみ Vue3 のサーバを許可オリジンに追加
    if cfg!(debug_assertions) {
        // 開発時は localhost:5173 でブラウザをオープンするため書き換え
        browser_url = "http://localhost:5173".to_string();
    }

    // TCPリスナー
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // サーバモードでなければブラウザ起動（-sオプションなしの場合）
    if !is_server_only {
        if webbrowser::open(&browser_url).is_ok() {
            tracing::info!("=================== Open Web Browser ===================");
        }
    }

    // コンソール出力オプション時
    if is_show_console {
        // Windows かつ サーバモードの場合はコンソール出力
        #[cfg(windows)]
        ensure_console();
    }

    tracing::info!("========== Listening on http://{} ==========", addr);

    // サーバ起動
    axum::serve(listener, app).await.unwrap();
}

// ルートへのアクセスは /index にリダイレクト
async fn root_handler() -> impl IntoResponse {
    Redirect::permanent("/index")
}

// アプリケーション初期設定情報の取得ハンドラ
async fn get_app_init_handler(_: Request<Body>) -> Json<AppInit> {
    Json(AppInit {
        app_title: CONFIG.app_title.clone(),
        allow_user_account_create: CONFIG.allow_user_create_account,
        allow_origins: CONFIG.allow_origins.clone(),
    })
}

// 死活監視用API
async fn health_check_handler() -> Json<MessageApi> {
    Json(MessageApi {
        message: "Hello, I'm administrator.".to_string(),
    })
}

// INDEX HTML GET HANDLER
async fn index_handler(headers: HeaderMap) -> Result<Html<String>, AppError> {
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
        },
        None => Err(AppError::NotFound),
    }
}

async fn licenses_get_handler() -> Result<Html<String>, AppError> {
    match Asset::get("licenses.html") {
        Some(content) => {
            let html_content = String::from_utf8(content.data.into_owned()).unwrap();
            Ok(Html(html_content))
        },
        None => Err(AppError::NotFound),
    }
}

// 404 HANDLER
async fn custom_not_found_handler(_: Request<Body>) -> impl IntoResponse {
    Redirect::permanent("/index")
}

async fn serve_favicon() -> Result<Response<Body>, AppError> {
    match Asset::get("assets/favicon.ico") {
        Some(content) => {
            let body = content.data.into_owned();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "image/x-icon")
                .body(body.into())
                .expect("Failed to construct response");
            Ok(response)
        },
        None => Err(AppError::NotFound),
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

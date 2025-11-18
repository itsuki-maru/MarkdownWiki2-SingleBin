use std::time::Duration;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::path::PathBuf;
use thiserror::Error;

// アプリケーション設定情報構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationInitSetup {
    pub app_title: String,
    pub sqlite_database_path: PathBuf,
    pub database_url: String,
    pub access_token_exp_minutes: String,
    pub refresh_token_exp_minutes: String,
    pub secret_key: String,
    pub admin_username: String,
    pub admin_passwotd: String,
    pub image_file_path: String,
    pub upload_file_path: String,
    pub failed_account_lock: String,
    pub next_challenge_minutes: String,
    pub challenge_limit_time_failed_count: String,
    pub cache_control: String,
    pub secure_cookie: String,
    pub service_name: String,
    pub rust_log: String,
    pub allow_user_create_account: String,
    pub allow_origins: String,
}

// ルートのメッセージ構造体
#[derive(Serialize, Deserialize)]
pub struct MessageApi {
    pub message: String,
}

// DB返り値の構造体の型
#[derive(Serialize, Deserialize)]
pub struct ReturningId {
    pub id: String,
}

// アプリケーション初期情報構造体
#[derive(Serialize, Deserialize)]
pub struct AppInit {
    pub app_title: String,
    pub allow_user_account_create: bool,
    pub allow_origins: String,
}

// Account用のユーザー構造体
#[derive(Serialize, Deserialize, FromRow)]
pub struct UserAccountModel {
    pub id: String,
    pub username: String,
    pub password: String,
    pub create_at: String,
    pub is_superuser: bool,
    pub failed_count: i64,
    pub next_challenge_time: String,
    pub is_locked: bool,
    pub is_private: bool,
    pub is_basic_authed: bool,
    pub is_basic_authed_at: String,
    pub totp_secret: String,
    pub totp_temp_secret: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct IsExists {
    pub exists_flag: i64,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct IsSuperuser {
    pub is_superuser: bool,
}

// サインアップ情報構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct SignupPayload {
    pub username: String,
    pub public_name: String,
    pub password: String,
}

// ログイン情報構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

// TOTPによるログイン情報構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct TotpLoginPayload {
    pub totp_token: String,
    pub user_id: String,
}

// TOTPセットアップ構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    pub otpauth_url: String,
    pub secret_base32: String,
}

// ユーザー名取得構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserNameFromDb {
    pub username: String,
}

// TOTP有効化リクエスト構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct TotpVerifyRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpTempSecret {
    pub totp_temp_secret: String,
}

// トークン構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub token_type: String,
    pub exp: usize,
    pub sub: String,
}

// アクセストークンとリフレッシュトークンの両者
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPare {
    pub access_token: String,
    pub refresh_token: String,
}


// ユーザー名とIDを返す構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
    pub public_name: String,
}

// アカウントプライバシー設定取得構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountPrivacyInfo {
    pub is_private: bool,
    pub totp_secret: String,
}

// アカウントのプライバシー設定構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccountPrivacyPayload {
    pub is_private: bool,
}

// Wikiデータ構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WikiData {
    pub id: String,
    pub user_id: String,
    pub date: String,
    pub title: String,
    pub body: String,
    pub update_at: String,
    pub is_public: bool,
}

// Wikiデータ構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWikiData {
    pub title: String,
    pub body: String,
    pub is_public: bool,
}

// レスポンス用Wikiデータ構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ResponseWikiData {
    pub id: String,
    pub user_id: String,
    pub date: String,
    pub title: String,
    pub body: String,
    pub update_at: String,
    pub is_public: bool,
}

// Wiki作成後のID返却
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseWikiId {
    pub message: String,
    pub user_id: String,
    pub new_wiki_id: String,
    pub date: String,
}

// Wiki更新データ構造体
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateWikiData {
    pub title: String,
    pub body: String,
    pub is_public: bool,
}

// Wikiダウンロードデータ構造体
#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadWikiData {
    pub title: String,
    pub body: String,
}

// Wiki更新後のレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatedWikiResponse {
    pub id: String,
    pub message: String,
}

// Wikiオーナー返却構造体
#[derive(Serialize, Deserialize)]
pub struct WikiOwner {
    pub id: String,
    pub username: String,
    pub public_name: String,
}

// 画像構造体
#[derive(Serialize, Deserialize, Debug)]
pub struct ImageData {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub uuid_filename: String,
}

// 画像削除後のデータベース構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageIdNameDeleted {
    pub id: String,
    pub uuid_filename: String,
}

// 画像削除後の構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct DeletedImageResponse {
    pub id: String,
    pub message: String,
}

// 画像アップロード後のレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponseImage {
    pub new_image_id: String,
    pub user_id: String,
    pub filename: String,
    pub uuid_filename: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WikiQueryParams {
    #[serde(default = "default_string")]
    pub query1: String,
    #[serde(default = "default_string")]
    pub query2: String,
}

// デフォルト値を指定するための関数
fn default_string() -> String {
    "".to_string()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseUserData {
    pub id: String,
    pub username: String,
    pub public_name: String,
    pub password: String,
    pub create_at: String,
    pub is_superuser: bool,
    pub is_locked: bool,
}

// ユーザーパスワード更新構造体
#[derive(Serialize, Deserialize)]
pub struct UpdateUserPasswordData {
    pub new_password: String,
}

// 公開ユーザー名更新構造体
#[derive(Serialize, Deserialize)]
pub struct UpdateUserPublicNameData {
    pub public_name: String,
}

// URL作成のエラーハンドリング
#[derive(Debug, Error)]
pub enum TempUrlError {
    #[error("Time error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),
    #[error("Duration overflow")]
    DurationOverflow,
}

// 一時URLと有効期限を保存するデータ構造
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TemporaryUrl {
    pub id: String,
    pub user_id: String,
    pub wiki_id: String,
    pub url: String,
    pub expiration: String,
    pub title: String,
    pub body: String,
    pub create_at: String,
}

impl TemporaryUrl {
    pub fn new(
        uuid: String,
        user_id: String,
        wiki_id: String,
        url: String,
        ttl: Duration,
        title: String,
        body: String,
        create_at: String
    ) -> Result<Self, TempUrlError> {
        let expiration = Utc::now().naive_utc()
            .checked_add_signed(
                chrono::Duration::from_std(ttl)
                    .map_err(|_| TempUrlError::DurationOverflow)?
            )
            .ok_or(TempUrlError::DurationOverflow)?.to_string();
        Ok(Self { id: uuid, user_id, wiki_id, url, expiration, title, body, create_at}) // idはデータベースで生成
    }

    pub fn is_expired(&self) -> bool {
        // SQLiteでの文字列から日付型に戻す
        let expiration = NaiveDateTime::parse_from_str(&self.expiration, "%Y-%m-%d %H:%M:%S");
        match expiration {
            Ok(exp) => {
                exp < Utc::now().naive_utc()
            },
            Err(_e) => {
                false
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenarateUrlSecondsPayload {
    pub minutes: u64,
}

// temporary_urlに登録するWikiのタイトルと内容
pub struct WikiTempDataTitleAndBody {
    pub id: String,
    pub title: String,
    pub body: String,
}

// 一時URL作成後のレスポンス構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedTemporaryUrlResponse {
    pub id: String,
    pub url: String,
    pub expiration: String,
    pub title: String,
}

// 発行済み一時URL取得構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct IssuedTemporaryUrls {
    pub id: String,
    pub user_id: String,
    pub wiki_id: String,
    pub url: String,
    pub expiration: String,
    pub title: String,
    pub create_at: String,
}
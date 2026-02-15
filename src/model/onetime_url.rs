use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::time::Duration;
use thiserror::Error;

// 一時URL作成後のレスポンス構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedTemporaryUrlResponse {
    pub id: String,
    pub url: String,
    pub expiration: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenarateUrlSecondsPayload {
    pub minutes: u64,
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
        create_at: String,
    ) -> Result<Self, TempUrlError> {
        let expiration = Utc::now()
            .naive_utc()
            .checked_add_signed(
                chrono::Duration::from_std(ttl).map_err(|_| TempUrlError::DurationOverflow)?,
            )
            .ok_or(TempUrlError::DurationOverflow)?
            .to_string();
        Ok(Self {
            id: uuid,
            user_id,
            wiki_id,
            url,
            expiration,
            title,
            body,
            create_at,
        }) // idはデータベースで生成
    }

    pub fn is_expired(&self) -> bool {
        // SQLiteでの文字列から日付型に戻す
        let expiration = NaiveDateTime::parse_from_str(&self.expiration, "%Y-%m-%d %H:%M:%S");
        match expiration {
            Ok(exp) => exp < Utc::now().naive_utc(),
            Err(_e) => false,
        }
    }
}

// temporary_urlに登録するWikiのタイトルと内容
pub struct WikiTempDataTitleAndBody {
    pub id: String,
    pub title: String,
    pub body: String,
}

// URL作成のエラーハンドリング
#[derive(Debug, Error)]
pub enum TempUrlError {
    #[error("Time error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),
    #[error("Duration overflow")]
    DurationOverflow,
}

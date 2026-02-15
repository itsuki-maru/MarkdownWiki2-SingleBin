use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Type;
use std::str::FromStr;

// 編集リクエスト構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EditWikiRequest {
    pub edit_request_title: String,
    pub edit_request_body: String,
    pub request_message: Option<String>,
    pub status: EditRequestStatus,
}

// 編集リクエストの状態管理（REJECT: 却下、REQUESTNOW: 返答待ち、DRAFT: 下書き）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum EditRequestStatus {
    #[sqlx(rename = "REJECT")]
    Reject,
    #[sqlx(rename = "REQUESTNOW")]
    RequestNow,
    #[sqlx(rename = "DRAFT")]
    Draft,
    #[sqlx(rename = "APPLIED")]
    Applied,
}

// オーナーからの結果（REJECT: 却下、APPLY: 適用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum EditResponseStatus {
    Reject,
    Apply,
}

impl FromStr for EditResponseStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "REJECT" => Ok(EditResponseStatus::Reject),
            "APPLY" => Ok(EditResponseStatus::Apply),
            _ => Err(format!("invalid EditRequestStatus: {}", s)),
        }
    }
}

// query_as! に渡す値を 明示的に &str にする
impl EditRequestStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            EditRequestStatus::Reject => "REJECT",
            EditRequestStatus::RequestNow => "REQUESTNOW",
            EditRequestStatus::Draft => "DRAFT",
            EditRequestStatus::Applied => "APPLIED",
        }
    }
}

impl FromStr for EditRequestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "REJECT" => Ok(EditRequestStatus::Reject),
            "REQUESTNOW" => Ok(EditRequestStatus::RequestNow),
            "DRAFT" => Ok(EditRequestStatus::Draft),
            "APPLIED" => Ok(EditRequestStatus::Applied),
            _ => Err(format!("invalid EditRequestStatus: {}", s)),
        }
    }
}

// 編集リクエスト一覧レスポンス用構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EditWikiListFromDb {
    pub id: String,
    pub wiki_owner_id: String,
    pub request_public_user_name: String,
    pub request_wiki_id: String,
    pub original_title: String,
    pub original_body: String,
    pub edit_request_title: String,
    pub edit_request_body: String,
    pub create_at: String,
    pub request_message: Option<String>,
    pub status: String, // DBからはString型で受け取り
}

// オーナーの承認・却下構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EditWikiOwnerRequest {
    pub id: String,
    pub reject: bool,
}

// オーナーの承認・却下の結果レスポンス構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EditWikiOwnerResponse {
    pub id: String,
    pub status: EditResponseStatus,
}

// 編集リクエストのステータス管理構造体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EditWikiStatusResponse {
    pub id: String,
    pub message: String,
    pub status: EditRequestStatus,
}

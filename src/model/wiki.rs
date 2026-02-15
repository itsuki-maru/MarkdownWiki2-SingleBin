use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Wikiデータ構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWikiData {
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

// Wiki更新後のレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatedWikiResponse {
    pub id: String,
    pub message: String,
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
    pub is_edit_request: bool,
}

// Wikiオーナー返却構造体
#[derive(Serialize, Deserialize)]
pub struct WikiOwner {
    pub id: String,
    pub username: String,
    pub public_name: String,
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

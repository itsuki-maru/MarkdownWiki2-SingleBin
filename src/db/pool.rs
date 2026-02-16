use crate::config::CONFIG;
use crate::error::AppError;
use crate::model::ReturningId;
use chrono::{TimeDelta, Utc};
use sqlx::Error;
use sqlx::query_as;
use sqlx::sqlite::SqlitePool;
use std::fs;
use std::path::Path;
use tracing::info;
use uuid::Uuid;

// データベース接続の確立
pub async fn setup_database_pool() -> Result<SqlitePool, Error> {
    if !Path::new(&CONFIG.database_path).exists() {
        info!("The SQLite database file does not exists so create it.");
        fs::File::create(&CONFIG.database_path).expect("Faild to create SQLite database file.");
        info!("The SQLite database created...Ok");
        let pool = SqlitePool::connect(&CONFIG.database_url).await?;
        run_migrations(&pool).await?;
        info!("The SQLite database migration...Ok");
        Ok(pool)
    } else {
        let pool = SqlitePool::connect(&CONFIG.database_url).await?;
        Ok(pool)
    }
}

// ユーザーを作成する共通関数
pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    public_name: &str,
    hashed_password: &str,
    is_superuser: bool,
) -> Result<ReturningId, AppError> {
    let now = Utc::now().naive_utc();
    let yesterday = match TimeDelta::try_days(1) {
        Some(one_day_delta) => now - one_day_delta,
        None => {
            tracing::error!("TimeDelta creation error.");
            return Err(AppError::InternalServerError);
        },
    };

    let new_user_id = Uuid::now_v7().to_string();
    let blank_text = "".to_string();

    let returning_user_id = query_as!(
        ReturningId,
        r#"
        INSERT INTO user_model (
            id,
            username,
            public_name,
            password,
            create_at,
            is_superuser,
            failed_count,
            next_challenge_time,
            is_locked,
            is_private,
            is_basic_authed,
            is_basic_authed_at,
            totp_secret,
            totp_temp_secret
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id
        "#,
        new_user_id,
        username,
        public_name,
        hashed_password,
        now,
        is_superuser,
        0,
        yesterday,
        false,
        true,
        false,
        yesterday,
        blank_text,
        blank_text,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to user create");
        AppError::Sqlx(e)
    })?;

    Ok(returning_user_id)
}

// データベースのマイグレーション
async fn run_migrations(pool: &SqlitePool) -> Result<(), Error> {
    let schemas = vec![
        r#"
        CREATE TABLE IF NOT EXISTS user_model (
            id TEXT PRIMARY KEY NOT NULL,
            username CHARACTER VARYING(256) NOT NULL UNIQUE,
            public_name CHARACTER VARYING(256) NOT NULL,
            password CHARACTER VARYING(256) NOT NULL,
            create_at TEXT NOT NULL,
            is_superuser BOOLEAN NOT NULL,
            failed_count INTEGER NOT NULL,
            next_challenge_time TEXT NOT NULL,
            is_locked BOOLEAN NOT NULL,
            is_private BOOLEAN NOT NULL,
            is_basic_authed BOOLEAN DEFAULT FALSE NOT NULL,
            is_basic_authed_at TEXT NOT NULL,
            totp_secret CHARACTER VARYING(256) NOT NULL,
            totp_temp_secret CHARACTER VARYING(256) NOT NULL
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS wiki_model (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            date TEXT NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            create_at TEXT NOT NULL,
            update_at TEXT NOT NULL,
            is_public BOOLEAN NOT NULL,
            is_edit_request BOOLEAN NOT NULL DEFAULT FALSE,
            FOREIGN KEY (user_id) REFERENCES user_model(id) ON DELETE CASCADE
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS image_model (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            uuid_filename TEXT NOT NULL,
            create_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES user_model(id) ON DELETE CASCADE
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS temporary_urls (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            wiki_id TEXT NOT NULL,
            url TEXT NOT NULL,
            expiration TEXT NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            create_at TEXT NOT NULL
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS edit_request_wiki_model (
            id TEXT PRIMARY KEY NOT NULL,
            wiki_owner_id UUID NOT NULL,
            request_user_id UUID NOT NULL,
            request_wiki_id UUID NOT NULL,
            edit_request_title CHARACTER VARYING(100) NOT NULL,
            edit_request_body CHARACTER VARYING NOT NULL,
            create_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
            request_message TEXT,
            status TEXT NOT NULL CHECK (status IN ('REJECT', 'REQUESTNOW', 'DRAFT', 'APPLIED')),
            FOREIGN KEY (wiki_owner_id) REFERENCES user_model(id) ON DELETE CASCADE,
            FOREIGN KEY (request_user_id) REFERENCES user_model(id) ON DELETE CASCADE,
            FOREIGN KEY (request_wiki_id) REFERENCES wiki_model(id) ON DELETE CASCADE
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS application_settings (
            id TEXT PRIMARY KEY NOT NULL,
            setting_key VARCHAR(255) NOT NULL UNIQUE,
            setting_value VARCHAR(255) NOT NULL,
            description TEXT,
            create_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    ];

    // トランザクションの開始
    let mut tx = pool.begin().await?;

    for schema in schemas {
        sqlx::query(schema).execute(&mut *tx).await?;
    }

    // トランザクションの終了
    tx.commit().await?;

    Ok(())
}

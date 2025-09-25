use crate::scheme::{
    Token,
    TokenPare,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::ErrorKind};
use chrono;
use crate::config::CONFIG;


// アクセストークン・リフレッシュトークン作成
pub fn create_token(user_id: &String, minutes: i64, token_type: String) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::try_minutes(minutes)
        .expect("Failed to create duration"))
        .expect("valid timestamp")
        .timestamp();

    let access_token = Token {
        token_type: token_type,
        exp: expiration as usize,
        sub: user_id.clone(),
    };

    encode(
        &Header::default(),
        &access_token,
        &EncodingKey::from_secret(CONFIG.secret_key.as_ref()),
    )
}

// アクセストークン検証
pub fn verify_access_token(token: &str) -> Result<Token, ErrorKind> {
    let validation = Validation::default();

    match decode::<Token>(
        token,
        &DecodingKey::from_secret(CONFIG.secret_key.as_ref()),
        &validation,
    ) {
        Ok(data) => Ok(data.claims), // 有効なトークン
        Err(err) => {
            if let ErrorKind::ExpiredSignature = err.kind() {
                // 期限切れでも `sub` を取得するためにデコードを試みる
                let decoded = decode::<Token>(
                    token,
                    &DecodingKey::from_secret(CONFIG.secret_key.as_ref()),
                    &Validation {
                        validate_exp: false, // 有効期限の検証を無視
                        ..Validation::default()
                    },
                );

                if let Ok(_data) = decoded {
                    return Err(ErrorKind::ExpiredSignature); // `sub` は取得できるので、後続処理で利用
                }
            }
            Err(ErrorKind::InvalidToken) // 不正なトークン
        }
    }
}

// 新しいアクセストークンを発行し、リフレッシュトークンを更新する
pub fn refresh_access_token(user_id: String) -> Result<TokenPare, jsonwebtoken::errors::Error> {
    let access_token = create_token(&user_id, CONFIG.access_token_exp_minutes, "access_token".to_string())?;
    let refresh_token = create_token(&user_id, CONFIG.refresh_token_exp_minutes, "refresh_token".to_string())?;
    let token_pare = TokenPare {
        access_token: access_token.clone(),
        refresh_token: refresh_token.clone(),
    };
    Ok(token_pare)
}

// リフレッシュトークン検証
pub fn verify_refresh_token(token: &str) -> Result<Token, jsonwebtoken::errors::Error> {
    decode::<Token>(
        token,
        &DecodingKey::from_secret(CONFIG.secret_key.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}
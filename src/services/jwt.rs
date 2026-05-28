use crate::traits::{BlacklistableClaims, TokenManager};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct DefaultJwtManager {
    secret_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: i64,
    pub jti: String,
    pub token_type: TokenType,
}

impl BlacklistableClaims for JwtClaims {
    fn jti(&self) -> &str {
        &self.jti
    }

    fn exp(&self) -> i64 {
        self.exp
    }
}

#[derive(Debug)]
pub enum JwtError {
    Encode(jsonwebtoken::errors::Error),
    Decode(jsonwebtoken::errors::Error),
    InvalidTokenType,
}

impl DefaultJwtManager {
    pub fn new<T: Into<String>>(secret_key: T) -> Self {
        Self {
            secret_key: secret_key.into(),
        }
    }
}

#[async_trait]
impl TokenManager for DefaultJwtManager {
    type Token = AuthTokens;
    type Claims = JwtClaims;
    type Error = JwtError;

    async fn generate(&self, user_id: &str) -> Result<Self::Token, Self::Error> {
        let access_exp = (Utc::now() + Duration::minutes(15)).timestamp();

        let refresh_exp = (Utc::now() + Duration::days(7)).timestamp();

        let access_claims = JwtClaims {
            sub: user_id.to_string(),
            exp: access_exp,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        let refresh_claims = JwtClaims {
            sub: user_id.to_string(),
            exp: refresh_exp,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(JwtError::Encode)?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(JwtError::Encode)?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            expires_in: access_exp as usize,
            token_type: "Bearer".to_string(),
        })
    }

    async fn verify(&self, token: &str) -> Result<Self::Claims, Self::Error> {
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(JwtError::Decode)
    }

    async fn refresh(&self, refresh_token: &str) -> Result<Self::Token, Self::Error> {
        let claims = self.verify(refresh_token).await?;

        match claims.token_type {
            TokenType::Refresh => {}
            _ => return Err(JwtError::InvalidTokenType),
        }

        let new_access_exp = (Utc::now() + Duration::minutes(15)).timestamp();

        let new_refresh_exp = (Utc::now() + Duration::days(7)).timestamp();

        let new_access_claims = JwtClaims {
            sub: claims.sub.clone(),
            exp: new_access_exp,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        let new_refresh_claims = JwtClaims {
            sub: claims.sub,
            exp: new_refresh_exp,
            jti: Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
        };

        let access_token = encode(
            &Header::default(),
            &new_access_claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(JwtError::Encode)?;

        let refresh_token = encode(
            &Header::default(),
            &new_refresh_claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(JwtError::Encode)?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            expires_in: new_access_exp as usize,
            token_type: "Bearer".to_string(),
        })
    }
}

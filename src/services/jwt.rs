use crate::traits::TokenManager;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

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
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug)]
pub enum JwtError {
    Encode(jsonwebtoken::errors::Error),
    Decode(jsonwebtoken::errors::Error),
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
        let access_exp = (Utc::now() + Duration::minutes(15)).timestamp() as usize;

        let refresh_exp = (Utc::now() + Duration::days(7)).timestamp() as usize;

        let access_claims = JwtClaims {
            sub: user_id.to_string(),
            exp: access_exp,
        };

        let refresh_claims = JwtClaims {
            sub: user_id.to_string(),
            exp: refresh_exp,
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
            expires_in: access_exp,
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

        let new_access_exp = (Utc::now() + Duration::minutes(15)).timestamp() as usize;

        let new_refresh_exp = (Utc::now() + Duration::days(7)).timestamp() as usize;

        let new_access_claims = JwtClaims {
            sub: claims.sub.clone(),
            exp: new_access_exp,
        };

        let new_refresh_claims = JwtClaims {
            sub: claims.sub,
            exp: new_refresh_exp,
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
            expires_in: new_access_exp,
            token_type: "Bearer".to_string(),
        })
    }
}

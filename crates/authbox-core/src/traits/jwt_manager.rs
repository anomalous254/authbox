use async_trait::async_trait;

#[async_trait]
pub trait TokenManager {
    type Token;
    type Claims: 'static;
    type Error;

    async fn generate(&self, user_id: &str) -> Result<Self::Token, Self::Error>;
    async fn verify(&self, token: &str) -> Result<Self::Claims, Self::Error>;
    async fn refresh(&self, refresh_token: &str) -> Result<Self::Token, Self::Error>;
}

#[async_trait]
pub trait TokenBlacklistStore {
    type Error;

    async fn is_blacklisted(&self, jti: &str) -> Result<bool, Self::Error>;
    async fn blacklist_token(&self, jti: &str, expires_at: i64) -> Result<bool, Self::Error>;
}

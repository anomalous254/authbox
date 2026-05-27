use async_trait::async_trait;

#[async_trait]
pub trait TokenManager {
    type Token;
    type Claims;
    type Error;

    async fn generate(&self, user_id: &str) -> Result<Self::Token, Self::Error>;
    async fn verify(&self, token: &str) -> Result<Self::Claims, Self::Error>;
    async fn refresh(&self, refresh_token: &str) -> Result<Self::Token, Self::Error>;
}

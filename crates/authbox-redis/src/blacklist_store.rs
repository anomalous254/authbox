use async_trait::async_trait;
use authbox_core::traits::TokenBlacklistStore;
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct RedisBlacklistStore {
    pub client: redis::Client,
}

impl RedisBlacklistStore {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[derive(Debug)]
pub enum RedisBlacklistError {
    Redis(redis::RedisError),
}

#[async_trait]
impl TokenBlacklistStore for RedisBlacklistStore {
    type Error = RedisBlacklistError;

    async fn is_blacklisted(&self, jti: &str) -> Result<bool, Self::Error> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(RedisBlacklistError::Redis)?;

        let exists: bool = conn.exists(jti).await.map_err(RedisBlacklistError::Redis)?;

        Ok(exists)
    }

    async fn blacklist_token(&self, jti: &str, expires_at: i64) -> Result<bool, Self::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        //SAFE TTL calculation
        if expires_at <= now {
            return Ok(true);
        }

        let ttl = (expires_at - now) as u64;

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(RedisBlacklistError::Redis)?;

        let _: () = conn
            .set_ex(jti, "1", ttl)
            .await
            .map_err(RedisBlacklistError::Redis)?;

        Ok(true)
    }
}

use async_trait::async_trait;
use authbox_core::traits::OneTimeTokenStore;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisOttStore {
    pub client: redis::Client,
}

impl RedisOttStore {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl OneTimeTokenStore for RedisOttStore {
    type Error = redis::RedisError;

    async fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), Self::Error> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.set_ex::<_, _, ()>(key, value, ttl_seconds).await?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, Self::Error> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let value: Option<String> = conn.get(key).await?;

        Ok(value)
    }

    async fn delete(&self, key: &str) -> Result<(), Self::Error> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        conn.del::<_, ()>(key).await?;

        Ok(())
    }
}

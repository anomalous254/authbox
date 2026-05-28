use async_trait::async_trait;

#[async_trait]
pub trait OneTimeTokenStore {
    async fn set(&self, key: &str, value: &str, ttl_seconds: u64);
    async fn get(&self, key: &str) -> Option<String>;
    async fn delete(&self, key: &str);
}

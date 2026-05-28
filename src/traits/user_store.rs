use super::auth_user::AuthUser;
use async_trait::async_trait;

#[async_trait]
pub trait UserStore<U: AuthUser> {
    type Error;

    async fn find_by_id(&self, user_id: &str) -> Option<U>;
    async fn find_by_email(&self, email: &str) -> Option<U>;
    async fn create_user(&self, email: String, pass_hash: String) -> Result<U, Self::Error>;
    async fn update_user(&self, user: U) -> Result<U, Self::Error>;
    async fn delete_user(&self, user_id: &str) -> Result<(), Self::Error>;
}

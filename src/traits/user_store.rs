use super::auth_user::AuthUser;
use async_trait::async_trait;

#[async_trait]
pub trait UserStore {
    type Error;
    type User: AuthUser;

    async fn find_by_id(&self, user_id: &str) -> Option<Self::User>;
    async fn find_by_email(&self, email: &str) -> Option<Self::User>;
    async fn create_user(
        &self,
        email: String,
        pass_hash: String,
    ) -> Result<Self::User, Self::Error>;
    async fn update_user(&self, user: Self::User) -> Result<Self::User, Self::Error>;
    async fn delete_user(&self, user_id: &str) -> Result<(), Self::Error>;
}

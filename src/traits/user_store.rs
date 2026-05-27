use super::auth_user::AuthUser;

pub trait UserStore<U: AuthUser> {
    fn find_by_id(&self, user_id: &str) -> Option<U>;
    fn find_by_email(&self, email: &str) -> Option<U>;
    fn create_user(&mut self, email: String, pass_hash: String) -> U;
}

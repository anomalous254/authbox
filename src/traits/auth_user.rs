/// Auth user model contracts
pub trait AuthUser {
    fn id(&self) -> String;
    fn email(&self) -> &str;
    fn password_hash(&self) -> &str;
}

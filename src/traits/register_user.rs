/// Register user  model contract
pub trait RegisterUserInput {
    fn email(&self) -> &str;
    fn password(&self) -> &str;
}

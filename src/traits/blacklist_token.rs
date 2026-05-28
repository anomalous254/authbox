pub trait BlacklistableClaims {
    fn jti(&self) -> &str;
    fn exp(&self) -> i64;
}

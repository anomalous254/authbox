use crate::traits::*;

pub struct AuthService<S, P, T> {
    pub store: S,
    pub hasher: P,
    pub tokens: T,
}

impl<S, P, T> AuthService<S, P, T> {
    /// Register a new user
    pub async fn register<U>(&mut self, email: String, password: String) -> U
    where
        U: AuthUser,
        S: UserStore<U>,
        P: PasswordHasher,
        T: TokenManager,
    {
        let hash = self.hasher.hash(&password);

        self.store.create_user(email, hash)
    }

    /// Login and return token if credentials are valid
    pub async fn login<U>(&self, email: &str, password: &str) -> Result<Option<T::Token>, T::Error>
    where
        U: AuthUser,
        S: UserStore<U>,
        P: PasswordHasher,
        T: TokenManager,
    {
        let user = match self.store.find_by_email(email) {
            Some(user) => user,
            None => return Ok(None),
        };

        let valid = self.hasher.verify(password, user.password_hash());

        if !valid {
            return Ok(None);
        }

        let token = self.tokens.generate(&user.id()).await?;

        Ok(Some(token))
    }
}

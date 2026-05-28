use crate::traits::*;

#[derive(Debug)]
pub enum AuthError<T, B> {
    Token(T),
    Blacklist(B),
    BlacklistedToken,
}

pub struct AuthService<S, P, T, B> {
    pub store: S,
    pub hasher: P,
    pub tokens: T,
    pub blacklist: B,
}

impl<S, P, T, B> AuthService<S, P, T, B> {
    /// Register a new user
    pub async fn register<U>(&mut self, email: String, password: String) -> U
    where
        U: AuthUser,
        S: UserStore<U>,
        P: PasswordHasher,
        T: TokenManager,
        B: TokenBlacklistStore,
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

    /// Refresh access token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<T::Token, AuthError<T::Error, B::Error>>
    where
        T: TokenManager,
        T::Claims: BlacklistableClaims,
        B: TokenBlacklistStore,
    {
        let claims = self
            .tokens
            .verify(refresh_token)
            .await
            .map_err(AuthError::Token)?;

        let blacklisted = self
            .blacklist
            .is_blacklisted(claims.jti())
            .await
            .map_err(AuthError::Blacklist)?;

        if blacklisted {
            return Err(AuthError::BlacklistedToken);
        }

        self.blacklist
            .blacklist_token(claims.jti(), claims.exp())
            .await
            .map_err(AuthError::Blacklist)?;

        self.tokens
            .refresh(refresh_token)
            .await
            .map_err(AuthError::Token)
    }
}

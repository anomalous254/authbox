use crate::configs::AuthServiceBuilder;
use crate::traits::*;
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthError<T, B> {
    Token(T),
    Blacklist(B),
    BlacklistedToken,
}

#[derive(Debug)]
pub enum LoginError<T, S> {
    InvalidCredentials,
    EmailNotVerified,
    Store(S),
    Token(T),
}

pub struct AuthService<S, P, T, B, E, M, V> {
    pub store: S,
    pub hasher: P,
    pub tokens: T,
    pub blacklist: B,
    pub email_sender: E,
    pub email_templates: M,
    pub ott_store: V,
}

impl<S, P, T, B, E, M, V> AuthService<S, P, T, B, E, M, V> {
    /// builder constructor
    pub fn builder() -> AuthServiceBuilder<S, P, T, B, E, M, V> {
        AuthServiceBuilder::new()
    }

    /// Register a new user
    pub async fn register(&mut self, input: S::RegisterDto) -> Result<S::User, S::Error>
    where
        S: UserStore,
        P: PasswordHasher,
        T: TokenManager,
        B: TokenBlacklistStore,
        E: EmailProvider,
        M: EmailTemplateConfig<S::User>,
        V: OneTimeTokenStore,
    {
        let hash = self.hasher.hash(input.password());
        let user = self.store.create_user(input, hash).await?;

        let token = Uuid::new_v4().to_string();

        self.ott_store.set(&token, &user.id(), 60 * 60 * 24).await;

        let subject = self.email_templates.verify_email_subject(&user);
        let body = self.email_templates.verify_email_body(&user, &token);

        let _ = self
            .email_sender
            .send_email(user.email(), &subject, &body)
            .await;

        Ok(user)
    }

    /// Login and return token if credentials are valid
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<T::Token, LoginError<T::Error, S::Error>>
    where
        S: UserStore,
        P: PasswordHasher,
        T: TokenManager,
    {
        let user = match self.store.find_by_email(email).await {
            Some(user) => user,
            None => return Err(LoginError::InvalidCredentials),
        };

        let verified = self
            .store
            .is_email_verified(&user.id())
            .await
            .map_err(LoginError::Store)?;

        if !verified {
            return Err(LoginError::EmailNotVerified);
        }

        let valid = self.hasher.verify(password, user.password_hash());

        if !valid {
            return Err(LoginError::InvalidCredentials);
        }

        self.tokens
            .generate(&user.id())
            .await
            .map_err(LoginError::Token)
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

    /// Verify email
    pub async fn verify_email(&self, token: &str)
    where
        S: UserStore,
        V: OneTimeTokenStore,
    {
        if let Some(user_id) = self.ott_store.get(token).await {
            let mut user = self.store.find_by_id(&user_id).await.unwrap();
            user.set_email_verified(true);
            let _ = self.store.update_user(user).await;

            self.ott_store.delete(token).await;
        }
    }

    /// request password reset
    pub async fn request_password_reset(&self, email: &str)
    where
        S: UserStore,
        V: OneTimeTokenStore,
        M: EmailTemplateConfig<S::User>,
        E: EmailProvider,
    {
        if let Some(user) = self.store.find_by_email(email).await {
            let token = Uuid::new_v4().to_string();

            self.ott_store.set(&token, &user.id(), 60 * 15).await;

            let subject = self.email_templates.reset_password_subject(&user);
            let body = self.email_templates.reset_password_body(&user, &token);

            let _ = self
                .email_sender
                .send_email(user.email(), &subject, &body)
                .await;
        }
    }

    /// reset password
    pub async fn reset_password(&self, token: &str, new_password: &str)
    where
        S: UserStore,
        V: OneTimeTokenStore,
        P: PasswordHasher,
    {
        if let Some(user_id) = self.ott_store.get(token).await
            && let Some(mut user) = self.store.find_by_id(&user_id).await
        {
            let password_hash = self.hasher.hash(new_password);
            user.set_password_hash(password_hash);

            let _ = self.store.update_user(user).await;

            self.ott_store.delete(token).await;
        }
    }
}

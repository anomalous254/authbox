use crate::configs::AuthServiceBuilder;
use crate::traits::*;
use std::marker::PhantomData;
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthError<T, B> {
    Token(T),
    Blacklist(B),
    BlacklistedToken,
}

pub struct AuthService<U, S, P, T, B, E, M, V> {
    pub store: S,
    pub hasher: P,
    pub tokens: T,
    pub blacklist: B,
    pub email_sender: E,
    pub email_templates: M,
    pub ott_store: V, //  [ one time token store ] for email verify and password reset

    pub _marker: PhantomData<U>,
}

impl<U, S, P, T, B, E, M, V> AuthService<U, S, P, T, B, E, M, V> {
    /// builder constructor
    pub fn builder() -> AuthServiceBuilder<S, P, T, B, E, M, V> {
        AuthServiceBuilder::new()
    }

    /// Register a new user
    pub async fn register(&mut self, email: String, password: String) -> Result<U, S::Error>
    where
        U: AuthUser,
        S: UserStore<U>,
        P: PasswordHasher,
        T: TokenManager,
        B: TokenBlacklistStore,
        E: EmailProvider,
        M: EmailTemplateConfig<U>,
        V: OneTimeTokenStore,
    {
        let hash = self.hasher.hash(&password);
        let user = self.store.create_user(email, hash).await?;

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
    pub async fn login(&self, email: &str, password: &str) -> Result<Option<T::Token>, T::Error>
    where
        U: AuthUser,
        S: UserStore<U>,
        P: PasswordHasher,
        T: TokenManager,
    {
        let user = match self.store.find_by_email(email).await {
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

    /// Veriy email
    pub async fn verify_email(&self, token: &str) -> ()
    where
        U: AuthUser,
        V: OneTimeTokenStore,
        S: UserStore<U>,
    {
        let user_id = self.ott_store.get(token).await;

        if let Some(user_id) = user_id {
            let mut user = self.store.find_by_id(&user_id).await.unwrap();
            user.set_email_verified(true);
            let _ = self.store.update_user(user).await;

            self.ott_store.delete(token).await;
        }
    }

    /// request user password reset
    pub async fn request_password_reset(&self, email: &str)
    where
        U: AuthUser,
        V: OneTimeTokenStore,
        S: UserStore<U>,
        M: EmailTemplateConfig<U>,
        E: EmailProvider,
    {
        if let Some(user) = self.store.find_by_email(email).await {
            let token = Uuid::new_v4().to_string();

            // 15 mins
            self.ott_store.set(&token, &user.id(), 60 * 15).await;

            let subject = self.email_templates.reset_password_subject(&user);

            let body = self.email_templates.reset_password_body(&user, &token);

            let _ = self
                .email_sender
                .send_email(user.email(), &subject, &body)
                .await;
        }
    }

    /// reset user password
    pub async fn reset_password(&self, token: &str, new_password: &str)
    where
        U: AuthUser,
        V: OneTimeTokenStore,
        S: UserStore<U>,
        P: PasswordHasher,
    {
        let user_id = self.ott_store.get(token).await;

        if let Some(user_id) = user_id
            && let Some(mut user) = self.store.find_by_id(&user_id).await
        {
            let password_hash = self.hasher.hash(new_password);

            user.set_password_hash(password_hash);

            let _ = self.store.update_user(user).await;

            self.ott_store.delete(token).await;
        }
    }
}

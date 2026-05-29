# authbox

A lightweight, modular, async-first authentication framework for Rust.

`authbox` provides a flexible authentication system built around traits, pluggable components, and Tokio-ready async APIs.

It is designed for applications that need customizable authentication logic without being locked into a specific database, framework, or storage backend.

---

# Features

- Secure password hashing with Argon2
- JWT access + refresh token authentication
- Refresh token rotation
- Refresh token revocation / blacklisting
- Email verification flow
- Password reset flow
- One-time token (OTT) support
- Fully async (`tokio`)
- Trait-driven architecture
- Pluggable storage backends
- Framework agnostic
- Test-friendly design
- Builder API for ergonomic setup

---

# Installation

```bash
cargo add authbox
```

---

# Quick Start

## Import Prelude

```rust
use authbox::prelude::*;
```

---

# 1. Create Your User Type

Your user model must implement the `AuthUser` trait.

```rust
#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub is_email_verified: bool,
}

impl AuthUser for User {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn password_hash(&self) -> &str {
        &self.password_hash
    }

    fn is_email_verified(&self) -> bool {
        self.is_email_verified
    }

    fn set_email_verified(&mut self, verified: bool) {
        self.is_email_verified = verified;
    }

    fn set_password_hash(&mut self, hash: String) {
        self.password_hash = hash;
    }
}
```

---

# 2. Create a User Store

`authbox` works with any backend:

- PostgreSQL
- MySQL
- SQLite
- MongoDB
- Redis
- In-memory stores
- Custom storage systems

Implement the `UserStore` trait:

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct UserStoreImpl {
    pub users: Arc<Mutex<HashMap<String, User>>>,
}

impl UserStoreImpl {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserStore for UserStoreImpl {
    type Error = String;
    type User = User;

    async fn find_by_id(&self, user_id: &str) -> Option<User> {
        let users = self.users.lock().unwrap();

        users.get(user_id).cloned()
    }

    async fn find_by_email(&self, email: &str) -> Option<User> {
        let users = self.users.lock().unwrap();

        users.values().find(|u| u.email == email).cloned()
    }

    async fn create_user(
        &self,
        email: String,
        pass_hash: String,
    ) -> Result<User, Self::Error> {
        let mut users = self.users.lock().unwrap();

        let user = User {
            id: email.clone(),
            email,
            password_hash: pass_hash,
            is_email_verified: false,
        };

        users.insert(user.id.clone(), user.clone());

        Ok(user)
    }

    async fn update_user(
        &self,
        user: User,
    ) -> Result<User, Self::Error> {
        let mut users = self.users.lock().unwrap();

        users.insert(user.id.clone(), user.clone());

        Ok(user)
    }

    async fn delete_user(
        &self,
        user_id: &str,
    ) -> Result<(), Self::Error> {
        let mut users = self.users.lock().unwrap();

        users.remove(user_id);

        Ok(())
    }
}
```

---

# 3. Create a Password Hasher

Use the built-in Argon2 hasher:

```rust
let hasher = DefaultHasher;
```

Or implement your own `PasswordHasher`.

---

# 4. Create a JWT Manager

Use the built-in JWT implementation:

```rust
let tokens = DefaultJwtManager::new("super-secret-key");
```

Or implement your own `TokenManager`.

---

# 5. Create a Token Blacklist Store

Used for refresh token revocation.

```rust
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MemoryBlacklistStore {
    pub tokens: Arc<Mutex<HashSet<String>>>,
}

impl MemoryBlacklistStore {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[derive(Debug)]
pub struct BlacklistError;

#[async_trait]
impl TokenBlacklistStore for MemoryBlacklistStore {
    type Error = BlacklistError;

    async fn is_blacklisted(
        &self,
        jti: &str,
    ) -> Result<bool, Self::Error> {
        let store = self.tokens.lock().unwrap();

        Ok(store.contains(jti))
    }

    async fn blacklist_token(
        &self,
        jti: &str,
        _expires_at: i64,
    ) -> Result<bool, Self::Error> {
        let mut store = self.tokens.lock().unwrap();

        store.insert(jti.to_string());

        Ok(true)
    }
}
```

---

# 6. Create an OTT Store

One-time token storage used for:

- Email verification
- Password reset flows

```rust
#[derive(Clone)]
pub struct MemoryOttStore {
    pub store: Arc<Mutex<HashMap<String, String>>>,
}

impl MemoryOttStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl OneTimeTokenStore for MemoryOttStore {
    async fn set(
        &self,
        key: &str,
        value: &str,
        _: u64,
    ) {
        let mut store = self.store.lock().unwrap();

        store.insert(key.to_string(), value.to_string());
    }

    async fn get(
        &self,
        key: &str,
    ) -> Option<String> {
        let store = self.store.lock().unwrap();

        store.get(key).cloned()
    }

    async fn delete(&self, key: &str) {
        let mut store = self.store.lock().unwrap();

        store.remove(key);
    }
}
```

---

# 7. Create an Email Provider

```rust
#[derive(Clone)]
pub struct MockEmailSender;

#[async_trait]
impl EmailProvider for MockEmailSender {
    type Error = ();

    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Self::Error> {
        println!("TO: {}", to);
        println!("SUBJECT: {}", subject);
        println!("BODY: {}", body);

        Ok(())
    }
}
```

---

# 8. Create Email Templates

```rust
#[derive(Clone)]
pub struct MockTemplates;

#[async_trait]
impl EmailTemplateConfig<User> for MockTemplates {
    fn verify_email_subject(&self, _: &User) -> String {
        "Verify Email".to_string()
    }

    fn verify_email_body(
        &self,
        _: &User,
        token: &str,
    ) -> String {
        format!("verify token: {}", token)
    }

    fn reset_password_subject(&self, _: &User) -> String {
        "Reset Password".to_string()
    }

    fn reset_password_body(
        &self,
        _: &User,
        token: &str,
    ) -> String {
        format!("reset token: {}", token)
    }
}
```

---

# 9. Build the Auth Service

```rust
let auth = AuthService::builder()
    .store(UserStoreImpl::new())
    .hasher(DefaultHasher)
    .tokens(DefaultJwtManager::new("secret"))
    .blacklist(MemoryBlacklistStore::new())
    .email_sender(MockEmailSender)
    .email_templates(MockTemplates)
    .ott_store(MemoryOttStore::new())
    .build();
```

---

# Register User

```rust
let user = auth
    .register(
        "user@mail.com".to_string(),
        "password123".to_string(),
    )
    .await?;

println!("User created: {}", user.email());
```

---

# Login User

```rust
let login = auth
    .login(
        "user@mail.com",
        "password123",
    )
    .await?;

if let Some(tokens) = login {
    println!("Access Token: {}", tokens.access_token);

    println!("Refresh Token: {}", tokens.refresh_token);
}
```

---

# Refresh Tokens

```rust
let new_tokens = auth
    .refresh_token(&refresh_token)
    .await?;

println!("{}", new_tokens.access_token);
```

Old refresh tokens are automatically blacklisted after rotation.

---

# Email Verification Flow

## Request Verification

Verification emails are automatically sent during registration.

## Verify Email

```rust
auth.verify_email(&token).await;
```

---

# Password Reset Flow

## Request Password Reset

```rust
auth.request_password_reset("user@mail.com")
    .await;
```

## Reset Password

```rust
auth.reset_password(
    &token,
    "new-password",
).await;
```

---

# Auth Token Structure

```rust
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
    pub token_type: String,
}
```

---

# Architecture

`authbox` is built around composable traits.

---

## AuthService

Handles authentication business logic:

- Registration
- Login
- Refresh token rotation
- Email verification
- Password reset

---

## UserStore

Handles persistence:

- Create user
- Find user
- Update user
- Delete user

---

## PasswordHasher

Handles password security:

- Hash passwords
- Verify passwords

---

## TokenManager

Handles JWT operations:

- Generate tokens
- Verify tokens
- Refresh tokens

---

## TokenBlacklistStore

Handles refresh token revocation.

---

## OneTimeTokenStore

Handles temporary token storage for:

- Email verification
- Password reset flows

---

## EmailProvider

Handles email delivery.

---

## EmailTemplateConfig

Handles customizable email templates.

---

# Recommended Production Stack

| Component | Recommendation |
|---|---|
| Database | PostgreSQL |
| ORM | SQLx or Diesel |
| Cache / Token Store | Redis |
| Runtime | Tokio |
| HTTP Framework | Axum or Actix |
| Password Hashing | Argon2 |

Redis is highly recommended for:

- Refresh token blacklisting
- One-time token storage
- TTL-based expiration
- Fast token lookup

---

# Roadmap

- [x] Async architecture
- [x] JWT authentication
- [x] Refresh token rotation
- [x] Refresh token revocation
- [x] Email verification
- [x] Password reset flow
- [x] Pluggable storage backends
- [ ] Axum middleware
- [ ] Actix middleware
- [ ] Session management

---

# License

Apache-2.0








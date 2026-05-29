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
- Custom registration DTO support

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

# 1. Create a Custom Registration DTO

`authbox` supports fully customizable registration DTOs.

The DTO name can be anything.

```rust
#[derive(Clone, Debug)]
pub struct SignupRequest {
    // required auth fields
    pub email: String,
    pub password: String,

    // custom application fields
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub age: Option<u32>,
    pub bio: Option<String>,
}
```

---

## Implement `RegisterUserInput`

`authbox` only requires access to:

- email
- password

```rust
impl RegisterUserInput for SignupRequest {
    fn email(&self) -> &str {
        &self.email
    }

    fn password(&self) -> &str {
        &self.password
    }
}
```

---

# 2. Create Your User Type

Your user model must implement the `AuthUser` trait.

```rust
#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub is_email_verified: bool,

    // custom fields
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub age: Option<u32>,
    pub bio: Option<String>,
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

# 3. Create a User Store

`authbox` works with any backend:

- PostgreSQL
- MySQL
- SQLite
- MongoDB
- Redis
- In-memory stores
- Custom storage systems

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

    // custom registration dto
    type RegisterDto = SignupRequest;

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
        input: SignupRequest,
        pass_hash: String,
    ) -> Result<User, Self::Error> {
        let mut users = self.users.lock().unwrap();

        let user = User {
            id: input.email.clone(),
            email: input.email,
            password_hash: pass_hash,
            is_email_verified: false,

            username: input.username,
            first_name: input.first_name,
            last_name: input.last_name,
            phone: input.phone,
            country: input.country,
            city: input.city,
            age: input.age,
            bio: input.bio,
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

# 4. Create a Password Hasher

Use the built-in Argon2 hasher:

```rust
let hasher = DefaultHasher;
```

Or implement your own `PasswordHasher`.

---

# 5. Create a JWT Manager

Use the built-in JWT implementation:

```rust
let tokens = DefaultJwtManager::new("super-secret-key");
```

Or implement your own `TokenManager`.

---

# 6. Build the Auth Service

```rust
let auth = AuthService::builder()
    .store(UserStoreImpl::new())
    .hasher(DefaultHasher)
    .tokens(DefaultJwtManager::new("secret"))
    .build();
```

---

# Register User

```rust
let user = auth
    .register(SignupRequest {
        email: "sarah.connor@example.com".to_string(),
        password: "UltraSecurePassword123!".to_string(),

        username: Some("sarahconnor".to_string()),
        first_name: Some("Sarah".to_string()),
        last_name: Some("Connor".to_string()),
        phone: Some("+1-555-0199".to_string()),
        country: Some("United States".to_string()),
        city: Some("Los Angeles".to_string()),
        age: Some(32),
        bio: Some(
            "Rust developer and cybersecurity enthusiast".to_string()
        ),
    })
    .await?;

println!("User created: {}", user.email());
```

---

# Login User

```rust
let login = auth
    .login(
        "sarah.connor@example.com",
        "UltraSecurePassword123!",
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
auth.request_password_reset(
    "sarah.connor@example.com",
)
.await;
```

## Reset Password

```rust
auth.reset_password(
    &token,
    "new-password",
)
.await;
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

## AuthService

Handles authentication business logic:

- Registration
- Login
- Refresh token rotation
- Email verification
- Password reset

## UserStore

Handles persistence:

- Create user
- Find user
- Update user
- Delete user

## RegisterUserInput

Defines the minimal registration fields required by the authentication system.

Required fields:

- email
- password

This enables fully customizable application-specific registration DTOs.

## PasswordHasher

Handles password security:

- Hash passwords
- Verify passwords

## TokenManager

Handles JWT operations:

- Generate tokens
- Verify tokens
- Refresh tokens

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

---

# Roadmap

- [x] Async architecture
- [x] JWT authentication
- [x] Refresh token rotation
- [x] Refresh token revocation
- [x] Email verification
- [x] Password reset flow
- [x] Pluggable storage backends
- [x] Custom registration DTO support
- [ ] Axum middleware
- [ ] Actix middleware
- [ ] Session management

---

# License

Apache-2.0

# authbox

A lightweight, modular authentication framework for Rust built around traits, async support, and pluggable components.

---

## It provides

- Password hashing (Argon2)
- JWT authentication (access + refresh tokens)
- Refresh token rotation
- Refresh token blacklisting / revocation
- Async-ready API (Tokio)
- Pluggable architecture
- Fully testable design

---

## Features

- User registration & login flow
- Secure password hashing using Argon2
- JWT access + refresh token support
- Refresh token rotation
- Refresh token revocation support
- Custom user store support (DB or in-memory)
- Custom token managers
- Custom password hashers
- Fully async (`tokio` + `async-trait`)
- Trait-based architecture for flexibility

---

## Installation

```bash
cargo add authbox
```

---

# Quick Start

## 1. Import prelude

```rust
use authbox::prelude::*;
```

---

## 2. Create JWT manager

You can use the default `DefaultJwtManager`
or create your own by implementing the `TokenManager` trait.

Example using the default implementation:

```rust
let tokens = DefaultJwtManager::new("my-secret-key");
```

---

## 3. Create password hasher

You can use the default `DefaultHasher`
or create your own by implementing the `PasswordHasher` trait.

Example using the default implementation:

```rust
let hasher = DefaultHasher;
```

---

## 4. Create a token blacklist store

The blacklist store is used for refresh token revocation.

You can use:

- Redis
- PostgreSQL
- SQLite
- In-memory storage
- Any custom backend

by implementing the `TokenBlacklistStore` trait.

Example in-memory implementation:

```rust
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct MemoryBlacklistStore {
    tokens: Arc<Mutex<HashSet<String>>>,
}

impl MemoryBlacklistStore {
    fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[derive(Debug)]
struct BlacklistError;

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

## 5. Implement a user type

Your custom user type must implement the `AuthUser` trait.

Example:

```rust
#[derive(Clone)]
struct YourCustomUserType {
    id: String,
    email: String,
    password_hash: String,

    // Other fields...
}

impl AuthUser for YourCustomUserType {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn password_hash(&self) -> &str {
        &self.password_hash
    }
}
```

---

## 6. Implement a user store

You can use:

- PostgreSQL
- MySQL
- SQLite
- MongoDB
- Redis
- In-memory storage

or any custom backend.

Your store must implement the `UserStore<U>` trait.

Example:

```rust
use std::collections::HashMap;

struct MyDbStore {
    users: HashMap<String, YourCustomUserType>,
}

impl MyDbStore {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl UserStore<YourCustomUserType> for MyDbStore {
    fn find_by_id(
        &self,
        user_id: &str,
    ) -> Option<YourCustomUserType> {
        self.users.get(user_id).cloned()
    }

    fn find_by_email(
        &self,
        email: &str,
    ) -> Option<YourCustomUserType> {
        self.users
            .values()
            .find(|u| u.email == email)
            .cloned()
    }

    fn create_user(
        &mut self,
        email: String,
        pass_hash: String,
    ) -> YourCustomUserType {
        let user = YourCustomUserType {
            id: email.clone(),
            email,
            password_hash: pass_hash,
        };

        self.users.insert(user.id.clone(), user.clone());

        user
    }
}
```

---

## 7. Create `AuthService`

```rust
let store = MyDbStore::new();

let tokens = DefaultJwtManager::new("my-secret-key");

let hasher = DefaultHasher;

let blacklist = MemoryBlacklistStore::new();

let mut auth = AuthService {
    store,
    hasher,
    tokens,
    blacklist,
};
```

---

# Register User

```rust
let user = auth
    .register::<YourCustomUserType>(
        "user@mail.com".to_string(),
        "password123".to_string(),
    )
    .await;

println!("User created: {}", user.email());
```

---

# Login User

```rust
let login = auth
    .login::<YourCustomUserType>(
        "user@mail.com",
        "password123",
    )
    .await?;

if let Some(tokens) = login {
    println!("Access token: {}", tokens.access_token);

    println!("Refresh token: {}", tokens.refresh_token);
}
```

---

# Refresh Tokens

```rust
let new_tokens = auth
    .refresh_token(&refresh_token)
    .await?;

println!(
    "New access token: {}",
    new_tokens.access_token
);
```

Old refresh tokens are automatically blacklisted after rotation.

---

# Token Structure

```rust
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
    pub token_type: String, // "Bearer"
}
```

---

# Architecture

`authbox` is built using traits for maximum flexibility.

---

## AuthService

Handles authentication business logic:

- register
- login
- refresh token rotation

---

## TokenManager

Handles JWT operations:

- generate
- verify
- refresh

---

## PasswordHasher

Handles password security:

- hash
- verify

---

## UserStore

Handles persistence:

- create user
- find user

---

## TokenBlacklistStore

Handles refresh token revocation:

- blacklist token
- check blacklist status

---

# Recommended Production Stack

```text
Redis is recommended for refresh token blacklisting because it supports automatic TTL expiration and very fast lookups.
```

---

# Roadmap

## Roadmap

- [x] Database adapters (SQLx, Diesel, MongoDB, and any backend storage supported via trait-based architecture)
- [x] Token blacklist / revocation system (pluggable via `TokenBlacklistStore`)
- [ ] RBAC (Role-based access control)
- [ ] OAuth2 integration
- [ ] Middleware for Axum / Actix
- [ ] Redis session support
- [ ] Email verification
- [ ] Password reset flow

---

# License

Apache-2.0








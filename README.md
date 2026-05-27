# authbox


A lightweight, modular authentication framework for Rust built around traits, async support, and pluggable components.

### It provides

- Password hashing (Argon2)
- JWT authentication (access + refresh tokens)
- Async-ready API (Tokio)
- Pluggable architecture (store, hasher, token manager)
- Fully testable design

### Features

- User registration & login flow
- Secure password hashing using Argon2
- JWT access + refresh token support
- Token refresh rotation
- Custom user store support (DB or in-memory)
- Fully async (tokio + async-trait)
- Trait-based architecture for flexibility

### Installation
```bash
```bash
cargo add authbox
```

#### Quick Start

1. Import prelude

```rust
use authbox::prelude::*;
```

2. Create JWT manager

**You can use the default `DefaultJwtManager` or use yours by implemting  the `TokenManager` trait**
 For example we use the `DefaultJwtManager`

```rust
let tokens = DefaultJwtManager::new("my-secret-key");
```

3. Create password hasher

**You can use the default `DefaultHasher` or use yours by implemting  the `PasswordHasher` trait**
 For example we use the `DefaultJwtManager`

```rust
let hasher = DefaultHasher;
```

4. Implement a user store

You can use your DB or in-memory store. and must implement the `UserStrore<U: YourCustomUserType>` and YourCustomUserType must implement the `AuthUser` trait;

Example:

```rust

  struct YourCustomUserType{
      id: String,
      email: String,
     password_hash: String,
    .... // Other fields
 }

  impl AuthUser for YourCustomUserType{
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


  struct MyDbStore;  // Must implemt the  >>>> UserStrore<U: YourCustomUserType>`

impl  UserStrore<YourCustomUserType> for MyDbStore{ 
  // add concrete implemetations
    fn find_by_id(&self, user_id: &str) -> Option<YourCustomUserType>;
    fn find_by_email(&self, email: &str) -> Option<YourCustomUserType>;
    fn create_user(&mut self, email: String, pass_hash: String) -> YourCustomUserType;
}

```

5. Create AuthService

```rust
let store = MyDbStore::new();
let tokens = DefaultJwtManager::new("my-secret-key");
let hasher = DefaultHasher;
let mut auth = AuthService {
    store,
    hasher,
    tokens,
};
```


6 Register User

```rust
let user = auth
    .register::<TestUser>(
        "user@mail.com".to_string(),
        "password123".to_string(),
    )
    .await;

println!("User created: {}", user.email());
}
```

7 Login User


```rust
let login = auth
    .login::<TestUser>("user@mail.com", "password123")
    .await?;

if let Some(tokens) = login {
    println!("Access token: {}", tokens.access_token);
    println!("Refresh token: {}", tokens.refresh_token);
}
```

8 Refresh Token


```rust
let new_tokens = auth
    .tokens
    .refresh(refresh_token)
    .await?;

println!("New access token: {}", new_tokens.access_token);

```

### Token Structure

```rust
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
    pub token_type: String, // "Bearer"
}
```


## Architecture

authbox is built using traits:

### AuthService

Handles business logic:

- register
- login

### TokenManager

Handles JWT:

- generate
- verify
- refresh

### PasswordHasher

Handles password security:

- hash
- verify

### UserStore

Handles persistence:

- create user
- find user




## Roadmap

- [ ] Database adapters (SQLx, MongoDB)
- [ ] Token revocation list
- [ ] Role-based access control (RBAC)
- [ ] OAuth2 integration
- [ ] Middleware for Actix / Axum
- [ ] Redis session support

## License

Apache-2.0












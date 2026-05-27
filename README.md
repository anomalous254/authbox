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

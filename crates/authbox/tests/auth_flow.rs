mod common;

use authbox::prelude::*;
use common::*;

#[test]
fn test_password_hashing() {
    let hasher = DefaultHasher;

    let hash = hasher.hash("password123");

    println!();
    println!("=== PASSWORD HASH ===");
    println!("{}", hash);

    assert!(hasher.verify("password123", &hash));
    assert!(!hasher.verify("wrong-password", &hash));
}

#[tokio::test]
async fn test_register_and_login_flow() {
    let mut auth = build_test_auth();

    // =========================
    // REGISTER
    // =========================

    let user = auth
        .register(RegisterDto {
            email: "john@test.com".to_string(),
            password: "password123".to_string(),
            username: Some("john".to_string()),
            phone: None,
            country: None,
            city: None,
            age: None,
        })
        .await
        .unwrap();

    println!();
    println!("=== REGISTER ===");
    println!("{:#?}", user);

    assert_eq!(user.email(), "john@test.com");

    // =========================
    // LOGIN BEFORE VERIFY
    // =========================

    let unverified = auth.login("john@test.com", "password123").await;

    println!();
    println!("=== LOGIN UNVERIFIED ===");
    println!("{:#?}", unverified);

    assert!(matches!(unverified, Err(LoginError::EmailNotVerified)));

    // =========================
    // VERIFY EMAIL
    // =========================

    let mut verified_user = auth.store.find_by_email("john@test.com").await.unwrap();

    verified_user.set_email_verified(true);

    auth.store.update_user(verified_user).await.unwrap();

    // =========================
    // LOGIN SUCCESS
    // =========================

    let tokens = auth.login("john@test.com", "password123").await.unwrap();

    println!();
    println!("=== LOGIN SUCCESS ===");
    println!("{:#?}", tokens);

    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());

    // =========================
    // LOGIN FAIL
    // =========================

    let failed = auth.login("john@test.com", "wrong-password").await;

    println!();
    println!("=== LOGIN FAILED ===");
    println!("{:#?}", failed);

    assert!(matches!(failed, Err(LoginError::InvalidCredentials)));
}

#[tokio::test]
async fn test_refresh_token_flow() {
    let mut auth = build_test_auth();

    auth.register(RegisterDto {
        email: "refresh@test.com".to_string(),
        password: "password123".to_string(),
        username: None,
        phone: None,
        country: None,
        city: None,
        age: None,
    })
    .await
    .unwrap();

    // =========================
    // VERIFY EMAIL
    // =========================

    let mut user = auth.store.find_by_email("refresh@test.com").await.unwrap();

    user.set_email_verified(true);

    auth.store.update_user(user).await.unwrap();

    // =========================
    // LOGIN
    // =========================

    let login = auth.login("refresh@test.com", "password123").await.unwrap();

    println!();
    println!("=== ORIGINAL TOKENS ===");
    println!("{:#?}", login);

    // =========================
    // REFRESH
    // =========================

    let refreshed = auth.refresh_token(&login.refresh_token).await;

    println!();
    println!("=== REFRESH RESULT ===");
    println!("{:#?}", refreshed);

    assert!(refreshed.is_ok());

    let new_tokens = refreshed.unwrap();

    assert!(!new_tokens.access_token.is_empty());
    assert!(!new_tokens.refresh_token.is_empty());
}

#[tokio::test]
async fn test_refresh_token_blacklist_flow() {
    let mut auth = build_test_auth();

    auth.register(RegisterDto {
        email: "blacklist@test.com".to_string(),
        password: "password123".to_string(),
        username: None,
        phone: None,
        country: None,
        city: None,
        age: None,
    })
    .await
    .unwrap();

    // =========================
    // VERIFY EMAIL
    // =========================

    let mut user = auth
        .store
        .find_by_email("blacklist@test.com")
        .await
        .unwrap();

    user.set_email_verified(true);

    auth.store.update_user(user).await.unwrap();

    // =========================
    // LOGIN
    // =========================

    let login = auth
        .login("blacklist@test.com", "password123")
        .await
        .unwrap();

    let refresh_token = login.refresh_token.clone();

    // =========================
    // FIRST REFRESH
    // =========================

    let first = auth.refresh_token(&refresh_token).await;

    println!();
    println!("=== FIRST REFRESH ===");
    println!("{:#?}", first);

    assert!(first.is_ok());

    // =========================
    // SECOND REFRESH
    // =========================

    let second = auth.refresh_token(&refresh_token).await;

    println!();
    println!("=== SECOND REFRESH ===");
    println!("{:#?}", second);

    assert!(matches!(second, Err(AuthError::BlacklistedToken)));
}

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
        .register("john@test.com".to_string(), "password123".to_string())
        .await
        .unwrap();

    println!();
    println!("=== REGISTER ===");
    println!("{:#?}", user);

    assert_eq!(user.email(), "john@test.com");

    // =========================
    // LOGIN SUCCESS
    // =========================

    let login = auth.login("john@test.com", "password123").await.unwrap();

    println!();
    println!("=== LOGIN SUCCESS ===");
    println!("{:#?}", login);

    assert!(login.is_some());

    let tokens = login.unwrap();

    assert!(!tokens.access_token.is_empty());

    assert!(!tokens.refresh_token.is_empty());

    // =========================
    // LOGIN FAIL
    // =========================

    let failed = auth.login("john@test.com", "wrong-password").await.unwrap();

    println!();
    println!("=== LOGIN FAILED ===");
    println!("{:#?}", failed);

    assert!(failed.is_none());
}

#[tokio::test]
async fn test_refresh_token_flow() {
    let mut auth = build_test_auth();

    auth.register("refresh@test.com".to_string(), "password123".to_string())
        .await
        .unwrap();

    let login = auth
        .login("refresh@test.com", "password123")
        .await
        .unwrap()
        .unwrap();

    println!();
    println!("=== ORIGINAL TOKENS ===");
    println!("{:#?}", login);

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

    auth.register("blacklist@test.com".to_string(), "password123".to_string())
        .await
        .unwrap();

    let login = auth
        .login("blacklist@test.com", "password123")
        .await
        .unwrap()
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
    // SHOULD FAIL
    // =========================

    let second = auth.refresh_token(&refresh_token).await;

    println!();
    println!("=== SECOND REFRESH ===");
    println!("{:#?}", second);

    assert!(matches!(second, Err(AuthError::BlacklistedToken)));
}

mod common;

use authbox::prelude::*;
use common::*;

#[tokio::test]
async fn test_email_verification_flow() {
    let mut auth = build_test_auth();

    // =========================
    // REGISTER
    // =========================

    let user = auth
        .register(RegisterDto {
            email: "verify@test.com".to_string(),
            password: "password123".to_string(),
            username: None,
            phone: None,
            country: None,
            city: None,
            age: None,
        })
        .await
        .unwrap();

    assert!(!user.is_email_verified());

    // =========================
    // GET VERIFICATION TOKEN
    // =========================

    let token = {
        let store = auth.ott_store.store.lock().unwrap();
        store.keys().next().unwrap().clone()
    };

    println!();
    println!("=== VERIFY TOKEN ===");
    println!("{}", token);

    // =========================
    // VERIFY EMAIL
    // =========================

    auth.verify_email(&token).await;

    // =========================
    // FETCH UPDATED USER
    // =========================

    let updated = auth.store.find_by_email("verify@test.com").await.unwrap();

    println!();
    println!("=== VERIFIED USER ===");
    println!("{:#?}", updated);

    assert!(updated.is_email_verified());
}

#[tokio::test]
async fn test_password_reset_flow() {
    let mut auth = build_test_auth();

    // =========================
    // REGISTER
    // =========================

    auth.register(RegisterDto {
        email: "reset@test.com".to_string(),
        password: "old-password".to_string(),
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

    let mut user = auth.store.find_by_email("reset@test.com").await.unwrap();

    user.set_email_verified(true);

    auth.store.update_user(user).await.unwrap();

    // =========================
    // REQUEST RESET
    // =========================

    auth.request_password_reset("reset@test.com").await;

    // =========================
    // GET RESET TOKEN
    // =========================

    let token = {
        let store = auth.ott_store.store.lock().unwrap();
        store.keys().next().unwrap().clone()
    };

    println!();
    println!("=== RESET TOKEN ===");
    println!("{}", token);

    // =========================
    // RESET PASSWORD
    // =========================

    auth.reset_password(&token, "new-password").await;

    // =========================
    // LOGIN WITH NEW PASSWORD
    // =========================

    let tokens = auth.login("reset@test.com", "new-password").await.unwrap();

    println!();
    println!("=== LOGIN AFTER RESET ===");
    println!("{:#?}", tokens);

    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());
}

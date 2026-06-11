mod common;

use authbox::prelude::*;
use common::*;
use redis::AsyncCommands;

/// Helper: fetch the first OTT token for a given user from Redis
async fn get_token(auth: &TestAuthService, user_id: &str) -> String {
    let mut conn = auth
        .ott_store
        .client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to get Redis connection");

    let keys: Vec<String> = redis::cmd("KEYS")
        .arg("*")
        .query_async(&mut conn)
        .await
        .expect("Failed to fetch keys from Redis");

    for key in keys {
        if let Ok(Some(value)) = conn.get::<_, Option<String>>(&key).await {
            if value == user_id {
                return key;
            }
        }
    }

    panic!("OTT token not found for user {}", user_id);
}

#[tokio::test]
async fn test_email_verification_flow() {
    let mut auth = build_test_auth();

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
    let token = get_token(&auth, "verify@test.com").await;

    println!("\n=== VERIFY TOKEN ===\n{}", token);

    // =========================
    // VERIFY EMAIL
    // =========================
    let result = auth.verify_email(&token).await;
    assert!(result.is_ok(), "First verification should succeed");

    // =========================
    // FETCH UPDATED USER
    // =========================
    let updated = auth.store.find_by_email("verify@test.com").await.unwrap();

    println!("\n=== VERIFIED USER ===\n{:#?}", updated);

    assert!(updated.is_email_verified());
}

#[tokio::test]
async fn test_password_reset_flow() {
    let mut auth = build_test_auth();

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
    // VERIFY EMAIL (manual)
    // =========================
    let mut user = auth.store.find_by_email("reset@test.com").await.unwrap();
    user.set_email_verified(true);
    auth.store.update_user(user).await.unwrap();

    // =========================
    // REQUEST RESET
    // =========================
    let _ = auth.request_password_reset("reset@test.com").await;

    // =========================
    // GET RESET TOKEN
    // =========================
    let token = get_token(&auth, "reset@test.com").await;

    println!("\n=== RESET TOKEN ===\n{}", token);

    // =========================
    // RESET PASSWORD
    // =========================
    let result = auth.reset_password(&token, "new-password").await;
    assert!(result.is_ok(), "Password reset should succeed");

    // =========================
    // LOGIN WITH NEW PASSWORD
    // =========================
    let tokens = auth.login("reset@test.com", "new-password").await.unwrap();

    println!("\n=== LOGIN AFTER RESET ===\n{:#?}", tokens);

    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());
}

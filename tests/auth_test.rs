use authbox::prelude::*;
use std::collections::HashMap;

// =========================
// TEST USER
// =========================
#[derive(Clone)]
#[allow(unused)]
struct TestUser {
    id: String,
    email: String,
    password_hash: String,

    username: String,
    is_active: bool,
    role: String,
    created_at: u64,
}

impl AuthUser for TestUser {
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

// =========================
// IN-MEMORY STORE
// =========================
struct TestStore {
    users: HashMap<String, TestUser>,
}

impl TestStore {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl UserStore<TestUser> for TestStore {
    fn find_by_id(&self, user_id: &str) -> Option<TestUser> {
        self.users.get(user_id).cloned()
    }

    fn find_by_email(&self, email: &str) -> Option<TestUser> {
        self.users.values().find(|u| u.email == email).cloned()
    }

    fn create_user(&mut self, email: String, pass_hash: String) -> TestUser {
        let user = TestUser {
            id: email.clone(),
            email,
            password_hash: pass_hash,
            username: "test_user".to_string(),
            is_active: true,
            role: "user".to_string(),
            created_at: 1234567890,
        };

        self.users.insert(user.id.clone(), user.clone());
        user
    }
}

// =========================
// TESTS
// =========================

#[test]
fn test_password_hashing() {
    let hasher = DefaultHasher;

    let hash = hasher.hash("password123");

    println!("\n=== PASSWORD HASH TEST ===");
    println!("hash: {}", hash);

    assert!(hasher.verify("password123", &hash));
    assert!(!hasher.verify("wrong", &hash));
}

// =========================
// AUTH FLOW TEST
// =========================

#[tokio::test]
async fn test_register_and_login_flow() {
    let store = TestStore::new();
    let hasher = DefaultHasher;
    let tokens = DefaultJwtManager::new("secret");

    let mut auth = AuthService {
        store,
        hasher,
        tokens,
    };

    // -------------------------
    // REGISTER
    // -------------------------
    let user = auth
        .register::<TestUser>("test@mail.com".to_string(), "password123".to_string())
        .await;

    println!("\n=== REGISTER ===");
    println!("id: {}", user.id());
    println!("email: {}", user.email());
    println!("role: {}", user.role);
    println!("active: {}", user.is_active);
    println!("created_at: {}", user.created_at);

    assert_eq!(user.email(), "test@mail.com");
    assert_eq!(user.role, "user");
    assert!(user.is_active);

    // -------------------------
    // LOGIN SUCCESS
    // -------------------------
    let login = auth.login::<TestUser>("test@mail.com", "password123").await;

    println!("\n=== LOGIN SUCCESS ===");
    println!("{:#?}", login);

    assert!(login.is_ok());
    assert!(login.unwrap().is_some());

    // -------------------------
    // LOGIN FAILURE
    // -------------------------
    let bad_login = auth
        .login::<TestUser>("test@mail.com", "wrongpassword")
        .await;

    println!("\n=== LOGIN FAIL ===");
    println!("{:#?}", bad_login);

    assert!(bad_login.is_ok());
    assert!(bad_login.unwrap().is_none());
}

#[tokio::test]
async fn test_refresh_token_flow() {
    use authbox::prelude::*;
    let store = TestStore::new();
    let hasher = DefaultHasher;
    let tokens = DefaultJwtManager::new("secret");

    let mut auth = AuthService {
        store,
        hasher,
        tokens,
    };

    // -------------------------
    // REGISTER USER
    // -------------------------
    let user = auth
        .register::<TestUser>("refresh@test.com".to_string(), "password123".to_string())
        .await;

    println!("\n=== REGISTER ===");
    println!("user id: {}", user.id());

    // -------------------------
    // LOGIN (GET TOKENS)
    // -------------------------
    let login = auth
        .login::<TestUser>("refresh@test.com", "password123")
        .await
        .unwrap()
        .unwrap();

    println!("\n=== LOGIN TOKENS ===");
    println!("access_token: {}", login.access_token);
    println!("refresh_token: {}", login.refresh_token);
    println!("expires_in: {}", login.expires_in);

    // -------------------------
    // REFRESH TOKEN FLOW
    // -------------------------
    let refreshed = auth.tokens.refresh(&login.refresh_token).await;

    println!("\n=== REFRESH RESULT ===");
    println!("{:#?}", refreshed);

    assert!(refreshed.is_ok());

    let new_tokens = refreshed.unwrap();

    println!("\n=== NEW TOKENS ===");
    println!("access_token: {}", new_tokens.access_token);
    println!("refresh_token: {}", new_tokens.refresh_token);
    println!("expires_in: {}", new_tokens.expires_in);

    assert!(!new_tokens.access_token.is_empty());
    assert!(!new_tokens.refresh_token.is_empty());
}

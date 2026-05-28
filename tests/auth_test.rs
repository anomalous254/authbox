use async_trait::async_trait;
use authbox::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

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
// IN-MEMORY USER STORE
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
// IN-MEMORY TOKEN BLACKLIST
// =========================

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

    async fn is_blacklisted(&self, jti: &str) -> Result<bool, Self::Error> {
        let store = self.tokens.lock().unwrap();

        Ok(store.contains(jti))
    }

    async fn blacklist_token(&self, jti: &str, _expires_at: i64) -> Result<bool, Self::Error> {
        let mut store = self.tokens.lock().unwrap();

        store.insert(jti.to_string());

        Ok(true)
    }
}

// =========================
// PASSWORD HASH TEST
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
// REGISTER + LOGIN TEST
// =========================

#[tokio::test]
async fn test_register_and_login_flow() {
    let store = TestStore::new();
    let hasher = DefaultHasher;
    let tokens = DefaultJwtManager::new("secret");
    let blacklist = MemoryBlacklistStore::new();

    let mut auth = AuthService {
        store,
        hasher,
        tokens,
        blacklist,
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

    assert_eq!(user.email(), "test@mail.com");

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

// =========================
// REFRESH TOKEN TEST
// =========================

#[tokio::test]
async fn test_refresh_token_flow() {
    let store = TestStore::new();
    let hasher = DefaultHasher;
    let tokens = DefaultJwtManager::new("secret");
    let blacklist = MemoryBlacklistStore::new();

    let mut auth = AuthService {
        store,
        hasher,
        tokens,
        blacklist,
    };

    // -------------------------
    // REGISTER
    // -------------------------

    auth.register::<TestUser>("refresh@test.com".to_string(), "password123".to_string())
        .await;

    // -------------------------
    // LOGIN
    // -------------------------

    let login = auth
        .login::<TestUser>("refresh@test.com", "password123")
        .await
        .unwrap()
        .unwrap();

    println!("\n=== LOGIN TOKENS ===");
    println!("access_token: {}", login.access_token);
    println!("refresh_token: {}", login.refresh_token);

    // -------------------------
    // REFRESH
    // -------------------------

    let refreshed = auth.tokens.refresh(&login.refresh_token).await;

    println!("\n=== REFRESH RESULT ===");
    println!("{:#?}", refreshed);

    assert!(refreshed.is_ok());

    let new_tokens = refreshed.unwrap();

    assert!(!new_tokens.access_token.is_empty());
    assert!(!new_tokens.refresh_token.is_empty());
}

// =========================
// TOKEN BLACKLIST TEST
// =========================

#[tokio::test]
async fn test_refresh_token_blacklist_flow() {
    let store = TestStore::new();
    let hasher = DefaultHasher;
    let tokens = DefaultJwtManager::new("secret");
    let blacklist = MemoryBlacklistStore::new();

    let mut auth = AuthService {
        store,
        hasher,
        tokens,
        blacklist,
    };

    // -------------------------
    // REGISTER
    // -------------------------

    auth.register::<TestUser>("blacklist@test.com".to_string(), "password123".to_string())
        .await;

    // -------------------------
    // LOGIN
    // -------------------------

    let login = auth
        .login::<TestUser>("blacklist@test.com", "password123")
        .await
        .unwrap()
        .unwrap();

    let refresh_token = login.refresh_token.clone();

    println!("\n=== ORIGINAL REFRESH TOKEN ===");
    println!("{}", refresh_token);

    // -------------------------
    // FIRST REFRESH
    // -------------------------

    let refreshed = auth.refresh_token(&refresh_token).await;

    println!("\n=== FIRST REFRESH ===");
    println!("{:#?}", refreshed);

    assert!(refreshed.is_ok());

    // -------------------------
    // SECOND REFRESH
    // SHOULD FAIL
    // -------------------------

    let second_try = auth.refresh_token(&refresh_token).await;

    println!("\n=== SECOND REFRESH ===");
    println!("{:#?}", second_try);

    assert!(matches!(second_try, Err(AuthError::BlacklistedToken)));
}


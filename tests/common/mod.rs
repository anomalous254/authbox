use async_trait::async_trait;
use authbox::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

// =========================
// REGISTER DTO (MANY FIELDS)
// =========================

#[derive(Clone, Debug)]
pub struct RegisterDto {
    pub email: String,
    pub password: String,

    pub username: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub age: Option<u32>,
}

// =========================
// TRAIT (ONLY AUTH CORE FIELDS)
// =========================

impl RegisterUserInput for RegisterDto {
    fn email(&self) -> &str {
        &self.email
    }

    fn password(&self) -> &str {
        &self.password
    }
}

// =========================
// USER MODEL
// =========================

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,

    pub is_email_verified: bool,

    pub username: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub age: Option<u32>,
}

// =========================
// AUTHUSER
// =========================

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

// =========================
// STORE
// =========================

#[derive(Clone)]
pub struct TestStore {
    pub users: Arc<Mutex<HashMap<String, TestUser>>>,
}

impl TestStore {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserStore for TestStore {
    type Error = String;
    type User = TestUser;
    type RegisterDto = RegisterDto;

    async fn find_by_id(&self, user_id: &str) -> Option<TestUser> {
        let users = self.users.lock().unwrap();
        users.get(user_id).cloned()
    }

    async fn find_by_email(&self, email: &str) -> Option<TestUser> {
        let users = self.users.lock().unwrap();
        users.values().find(|u| u.email == email).cloned()
    }

    async fn create_user(
        &self,
        input: RegisterDto,
        pass_hash: String,
    ) -> Result<TestUser, Self::Error> {
        let mut users = self.users.lock().unwrap();

        let user = TestUser {
            id: input.email.clone(),
            email: input.email,
            password_hash: pass_hash,
            is_email_verified: false,

            username: input.username,
            phone: input.phone,
            country: input.country,
            city: input.city,
            age: input.age,
        };

        users.insert(user.id.clone(), user.clone());

        Ok(user)
    }

    async fn update_user(&self, user: TestUser) -> Result<TestUser, Self::Error> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id.clone(), user.clone());
        Ok(user)
    }

    async fn delete_user(&self, user_id: &str) -> Result<(), Self::Error> {
        let mut users = self.users.lock().unwrap();
        users.remove(user_id);
        Ok(())
    }

    async fn is_email_verified(&self, user_id: &str) -> Result<bool, Self::Error> {
        let users = self.users.lock().unwrap();

        match users.get(user_id) {
            Some(user) => Ok(user.is_email_verified),
            None => Err("user not found".to_string()),
        }
    }
}

// =========================
// BLACKLIST STORE
// =========================

#[derive(Clone)]
pub struct MemoryBlacklistStore {
    pub tokens: Arc<Mutex<HashSet<String>>>,
}

impl MemoryBlacklistStore {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[derive(Debug)]
pub struct BlacklistError;

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
// OTT STORE
// =========================

#[derive(Clone)]
pub struct MemoryOttStore {
    pub store: Arc<Mutex<HashMap<String, String>>>,
}

impl MemoryOttStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl OneTimeTokenStore for MemoryOttStore {
    async fn set(&self, key: &str, value: &str, _: u64) {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), value.to_string());
    }

    async fn get(&self, key: &str) -> Option<String> {
        let store = self.store.lock().unwrap();
        store.get(key).cloned()
    }

    async fn delete(&self, key: &str) {
        let mut store = self.store.lock().unwrap();
        store.remove(key);
    }
}

// =========================
// EMAIL PROVIDER
// =========================

#[derive(Clone)]
pub struct MockEmailSender;

#[async_trait]
impl EmailProvider for MockEmailSender {
    type Error = ();

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Self::Error> {
        println!("\nEMAIL TO: {}\nSUBJECT: {}\nBODY: {}", to, subject, body);
        Ok(())
    }
}

// =========================
// EMAIL TEMPLATES
// =========================

#[derive(Clone)]
pub struct MockTemplates;

#[async_trait]
impl EmailTemplateConfig<TestUser> for MockTemplates {
    fn verify_email_subject(&self, _: &TestUser) -> String {
        "Verify Email".to_string()
    }

    fn verify_email_body(&self, _: &TestUser, token: &str) -> String {
        format!("verify token: {}", token)
    }

    fn reset_password_subject(&self, _: &TestUser) -> String {
        "Reset Password".to_string()
    }

    fn reset_password_body(&self, _: &TestUser, token: &str) -> String {
        format!("reset token: {}", token)
    }
}

// =========================
// TEST AUTH SERVICE TYPE
// =========================

type TestAuthService = AuthService<
    TestStore,
    DefaultHasher,
    DefaultJwtManager,
    MemoryBlacklistStore,
    MockEmailSender,
    MockTemplates,
    MemoryOttStore,
>;

pub fn build_test_auth() -> TestAuthService {
    AuthService::builder()
        .store(TestStore::new())
        .hasher(DefaultHasher)
        .tokens(DefaultJwtManager::new("secret"))
        .blacklist(MemoryBlacklistStore::new())
        .email_sender(MockEmailSender)
        .email_templates(MockTemplates)
        .ott_store(MemoryOttStore::new())
        .build()
}

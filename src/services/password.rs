use crate::traits::PasswordHasher;

use argon2::{
    Argon2, PasswordVerifier,
    password_hash::{PasswordHash, PasswordHasher as _, SaltString},
};

use rand_core::OsRng;

pub struct DefaultHasher;

impl PasswordHasher for DefaultHasher {
    fn hash(&self, password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("hashing failed")
            .to_string()
    }

    fn verify(&self, password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

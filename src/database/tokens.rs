use std::collections::HashMap;

use argon2::Argon2;
use argon2::PasswordHasher;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use sqlx::{query, query_as};
use zeroize::Zeroizing;

use crate::StdResult;
use crate::database::Database;
use crate::database::serial_number::SerialNumber;
use crate::errors::SonataDbError;

#[derive(Debug, Clone)]
/// A [HashMap] mapping a [SerialNumber] to a [String] token.
/// Only allows access to the inner store via methods implemented
/// on this type for reasons of additional data consistency and security
/// guarantees, that can only be provided this way. Implements [Zeroize] and [ZeroizeOnDrop]
/// on all values (not keys!) of the HashMap, ensuring no token is left in memory
/// after the application exits.
pub struct TokenStore {
    /// Inner store `s`
    s: HashMap<SerialNumber, String>,
    /// An owned database connection, for convenience
    p: Database,
}

impl TokenStore {
    pub async fn get_token(
        serial_number: &SerialNumber,
    ) -> Result<Zeroizing<String>, SonataDbError> {
        query!(
            // THIS IS WRONG I GOTTA REDO IT
            "SELECT id WHERE i.serial_number = $1 FROM idcsr as i LEFT JOIN idcert AS c ON i.id JOIN user_tokens AS u ON i.serial_number"
        )
    }
}

impl zeroize::Zeroize for TokenStore {
    fn zeroize(&mut self) {
        for (_serial_number, token) in self.s.iter_mut() {
            token.zeroize();
        }
    }
}

impl zeroize::ZeroizeOnDrop for TokenStore {}

/// DOCUMENTME
pub fn hash_auth_token(auth_token: &str) -> StdResult<String> {
    let argon_hasher = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    Ok(argon_hasher
        .hash_password(auth_token.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string())
}

pub(crate) async fn valid_token_in_db(db: Database, token: &str) -> crate::errors::SonataDbError {
    todo!()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use argon2::{PasswordHash, PasswordVerifier};

    use super::*;

    #[test]
    fn eq_tokens() {
        let token = "hi!ilovetheworld";
        let hash = hash_auth_token(token).unwrap();
        let pw_hash = PasswordHash::new(&hash).unwrap();
        Argon2::default().verify_password(token.as_bytes(), &pw_hash).unwrap();
    }
}

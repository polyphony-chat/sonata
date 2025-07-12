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
    /// For a given [SerialNumber], get the hash of the **latest**, active auth token from the database,
    /// if exists. As implied, will return `None` if there is no token in the database where
    /// `valid_not_after` is smaller than the current system timestamp.
    pub async fn get_valid_token(
        &self,
        serial_number: &SerialNumber,
    ) -> Result<Option<Zeroizing<String>>, SonataDbError> {
        let record = query!(
            r#"
                WITH csr_id AS (
                    -- Get the id from idcsr for the given numeric value
                    SELECT id 
                    FROM idcsr 
                    WHERE serial_number = $1
                ),
                valid_cert AS (
                    -- Check if this id exists in idcert
                    SELECT c.id
                    FROM csr_id c
                    WHERE EXISTS (
                        SELECT 1 
                        FROM idcert ic 
                        WHERE ic.idcsr_id = c.id
                    )
                )
                -- Query user_tokens and select the token with the largest valid_not_after
                SELECT ut.token_hash
                FROM valid_cert vc
                JOIN user_tokens ut ON ut.cert_id = vc.id
                WHERE ut.valid_not_after >= NOW() -- only return non-expired tokens
                ORDER BY ut.valid_not_after DESC NULLS LAST
                LIMIT 1;
            "#,
            serial_number.as_bigdecimal()
        )
        .fetch_optional(&self.p.pool)
        .await?;
        match record {
            Some(record) => Ok(Some(record.token_hash.into())),
            None => Ok(None),
        }
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

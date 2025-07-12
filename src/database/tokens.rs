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
    /// Create a new TokenStore with the given database connection.
    pub fn new(database: Database) -> Self {
        Self { s: HashMap::new(), p: database }
    }

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
                WHERE (ut.valid_not_after >= NOW() OR ut.valid_not_after IS NULL) -- only return non-expired tokens
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
    use std::str::FromStr;

    use argon2::{PasswordHash, PasswordVerifier};
    use bigdecimal::BigDecimal;
    use sqlx::{Pool, Postgres};

    use super::*;

    #[test]
    fn eq_tokens() {
        let token = "hi!ilovetheworld";
        let hash = hash_auth_token(token).unwrap();
        let pw_hash = PasswordHash::new(&hash).unwrap();
        Argon2::default().verify_password(token.as_bytes(), &pw_hash).unwrap();
    }

    #[sqlx::test(fixtures("../../fixtures/test_setup.sql"))]
    async fn test_get_valid_token_with_valid_token(pool: Pool<Postgres>) {
        let db = Database { pool };
        let token_store = TokenStore::new(db);

        // Test user 1 who has a valid token
        let serial_number =
            SerialNumber::from(BigDecimal::from_str("12345678901234567890").unwrap());
        let result = token_store.get_valid_token(&serial_number).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "valid_token_hash_1");
    }

    #[sqlx::test(fixtures("../../fixtures/test_setup.sql"))]
    async fn test_get_valid_token_with_multiple_tokens_returns_latest(pool: Pool<Postgres>) {
        let db = Database { pool };
        let token_store = TokenStore::new(db);

        // Test user 2 who has multiple valid tokens - should return the one with latest expiration
        let serial_number =
            SerialNumber::from(BigDecimal::from_str("98765432109876543210").unwrap());
        let result = token_store.get_valid_token(&serial_number).await.unwrap();

        assert!(result.is_some());
        // Should return the token with the longer expiration (2 hours, not 30 minutes)
        assert_eq!(result.unwrap().as_str(), "valid_token_hash_2");
    }

    #[sqlx::test(fixtures("../../fixtures/test_setup.sql"))]
    async fn test_get_valid_token_with_no_cert_returns_none(pool: Pool<Postgres>) {
        let db = Database { pool };
        let token_store = TokenStore::new(db);

        // Test user 3 who has no certificate (so no valid tokens)
        let serial_number =
            SerialNumber::from(BigDecimal::from_str("11111111111111111111").unwrap());
        let result = token_store.get_valid_token(&serial_number).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test(fixtures("../../fixtures/test_setup.sql"))]
    async fn test_get_valid_token_with_nonexistent_serial_returns_none(pool: Pool<Postgres>) {
        let db = Database { pool };
        let token_store = TokenStore::new(db);

        // Test with a serial number that doesn't exist
        let serial_number =
            SerialNumber::from(BigDecimal::from_str("99999999999999999999").unwrap());
        let result = token_store.get_valid_token(&serial_number).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test(fixtures("../../fixtures/test_setup.sql"))]
    async fn test_get_valid_token_excludes_expired_tokens(pool: Pool<Postgres>) {
        // Insert a user with only expired tokens
        sqlx::query!(
            "INSERT INTO actors (uaid, local_name, deactivated, joined) VALUES
            ('00000000-0000-0000-0000-000000000004', 'test_user_4', false, NOW())"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO public_keys (id, uaid, pubkey, algorithm_identifier) VALUES
            (4, '00000000-0000-0000-0000-000000000004', 'test_pubkey_4', 1)"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO idcsr (
                id, serial_number, uaid, actor_public_key_id, actor_signature,
                session_id, valid_not_before, valid_not_after, extensions, pem_encoded
            ) VALUES
            (4, 22222222222222222222, '00000000-0000-0000-0000-000000000004', 4, 'test_signature_4',
             'test_session_4', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_4', 'test_csr_pem_4')"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO idcert (
                idcsr_id, issuer_info_id, valid_not_before, valid_not_after,
                home_server_public_key_id, home_server_signature, pem_encoded
            ) VALUES
            (4, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 1, 'test_home_server_sig_4', 'test_cert_pem_4')"
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert only expired tokens
        sqlx::query!(
            "INSERT INTO user_tokens (token_hash, cert_id, uaid, valid_not_after) VALUES
            ('expired_token_hash_4_1', 4, '00000000-0000-0000-0000-000000000004', NOW() - INTERVAL '2 hours'),
            ('expired_token_hash_4_2', 4, '00000000-0000-0000-0000-000000000004', NOW() - INTERVAL '1 hour')"
        )
        .execute(&pool)
        .await
        .unwrap();

        let db = Database { pool };
        let token_store = TokenStore::new(db);
        let serial_number =
            SerialNumber::from(BigDecimal::from_str("22222222222222222222").unwrap());
        let result = token_store.get_valid_token(&serial_number).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test(fixtures("../../fixtures/test_setup.sql"))]
    async fn test_get_valid_token_with_null_expiration(pool: Pool<Postgres>) {
        // Insert a user with a token that has NULL valid_not_after
        sqlx::query!(
            "INSERT INTO actors (uaid, local_name, deactivated, joined) VALUES
            ('00000000-0000-0000-0000-000000000005', 'test_user_5', false, NOW())"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO public_keys (id, uaid, pubkey, algorithm_identifier) VALUES
            (5, '00000000-0000-0000-0000-000000000005', 'test_pubkey_5', 1)"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO idcsr (
                id, serial_number, uaid, actor_public_key_id, actor_signature,
                session_id, valid_not_before, valid_not_after, extensions, pem_encoded
            ) VALUES
            (5, 33333333333333333333, '00000000-0000-0000-0000-000000000005', 5, 'test_signature_5',
             'test_session_5', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_5', 'test_csr_pem_5')"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query!(
            "INSERT INTO idcert (
                idcsr_id, issuer_info_id, valid_not_before, valid_not_after,
                home_server_public_key_id, home_server_signature, pem_encoded
            ) VALUES
            (5, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 1, 'test_home_server_sig_5', 'test_cert_pem_5')"
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert a token with NULL valid_not_after (should be treated as never expiring)
        sqlx::query!(
            "INSERT INTO user_tokens (token_hash, cert_id, uaid, valid_not_after) VALUES
            ('never_expires_token_hash', 5, '00000000-0000-0000-0000-000000000005', NULL)"
        )
        .execute(&pool)
        .await
        .unwrap();

        let db = Database { pool };
        let token_store = TokenStore::new(db);
        let serial_number =
            SerialNumber::from(BigDecimal::from_str("33333333333333333333").unwrap());
        let result = token_store.get_valid_token(&serial_number).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "never_expires_token_hash");
    }
}

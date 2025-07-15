use rand::distr::{Alphanumeric, SampleString};
use sqlx::{query, query_as, types::Uuid};
use zeroize::Zeroizing;

use crate::{
	database::{Database, serial_number::SerialNumber},
	errors::SonataDbError,
};

#[derive(Debug, Clone)]
/// A [HashMap] mapping a [SerialNumber] to a [String] token.
/// Only allows access to the inner store via methods implemented
/// on this type for reasons of additional data consistency and security
/// guarantees, that can only be provided this way. Implements [Zeroize] and
/// [ZeroizeOnDrop] on all values (not keys!) of the HashMap, ensuring no token
/// is left in memory after the application exits.
pub struct TokenStore {
	/// An owned database connection, for convenience
	p: Database,
}

/// A pair of an API access token and a unique actor identifier (uaid), where
/// the access token belongs to that actor. Does not distinguish between
/// different clients/sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenActorIdPair {
	/// API access token
	pub token: Zeroizing<String>,
	/// Unique Actor Identifier (uaid), unique per local actor.
	pub uaid: Uuid,
}

impl TokenStore {
	/// Create a new TokenStore with the given database connection.
	pub fn new(database: Database) -> Self {
		Self { p: database }
	}

	/// For a given [SerialNumber], get the hash of the **latest**, active auth
	/// token from the database, if exists. As implied, will return `None` if
	/// there is no token in the database where `valid_not_after` is smaller
	/// than the current system timestamp.
	pub async fn get_token_userid(
		&self,
		serial_number: &SerialNumber,
	) -> Result<Option<TokenActorIdPair>, SonataDbError> {
		let record = query_as!(
            TokenActorIdPair,
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
                SELECT ut.token_hash AS token, ut.uaid AS uaid
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
			Some(record) => Ok(Some(TokenActorIdPair { token: record.token, uaid: record.uaid })),
			None => Ok(None),
		}
	}

	/// Given a `token_hash`, find out the `serial_number` of the `IdCert` of
	/// the user, who this `token_hash` is for.
	pub async fn get_token_serial_number(
		&self,
		token_hash: &str,
	) -> Result<Option<SerialNumber>, SonataDbError> {
		Ok(query!(
			"SELECT idcsr.serial_number
                FROM user_tokens
                JOIN idcert ON user_tokens.cert_id = idcert.idcsr_id
                JOIN idcsr ON idcert.idcsr_id = idcsr.id
                WHERE user_tokens.token_hash = $1;
            ",
			token_hash
		)
		.fetch_optional(&self.p.pool)
		.await?
		.map(|record| record.serial_number.into()))
	}

	/// Generate a CSPRNG generated alphanumerical token, suitable for
	/// authentication purposes, hash it, then upsert (insert or update, if
	/// exists) the token hash into the database.
	///
	/// ## Returns
	///
	/// Returns the token hash, if the operation was successful.
	///
	/// ## Errors
	///
	/// - If the `uaid` does not refer to an existing actor in the `actors`
	///   table
	/// - If the `cert_id` is `Some()`, but does not refer to a cert that is
	///   stored in the `idcert` table
	/// - If the database connection is bad
	pub async fn generate_upsert_token(
		&self,
		actor_id: &Uuid,
		cert_id: Option<i64>,
	) -> Result<String, SonataDbError> {
		let token_hash = hash_auth_token(&Alphanumeric.sample_string(&mut rand::rng(), 96));
		query!(
			"INSERT INTO user_tokens (token_hash, uaid, cert_id) VALUES ($1, $2, $3) ON CONFLICT (cert_id, uaid) DO UPDATE SET token_hash = EXCLUDED.token_hash",
			&token_hash,
			actor_id,
			cert_id
		)
		.execute(&self.p.pool)
		.await?;
		Ok(token_hash)
	}
}

impl zeroize::ZeroizeOnDrop for TokenStore {}

/// Hashes an auth token using a deterministic hash function (currently:
/// blake3), then returns the hash as a string.
pub fn hash_auth_token(auth_token: &str) -> String {
	blake3::hash(auth_token.as_bytes()).to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
	use std::str::FromStr;

	use bigdecimal::BigDecimal;
	use sqlx::{Pool, Postgres};

	use super::*;

	#[test]
	fn eq_tokens() {
		let token = "hi!ilovetheworld";
		let hash = hash_auth_token(token);

		let hash2 = hash_auth_token(token);
		assert_eq!(hash, hash2, "Same token should produce identical hashes");

		let different_token = "different_token";
		let different_hash = hash_auth_token(different_token);
		assert_ne!(hash, different_hash, "Different tokens should produce different hashes");

		assert!(!hash.is_empty(), "Hash should not be empty");
		assert_eq!(hash.len(), 64, "Blake3 hash should be 64 characters long (256 bits as hex)");
		assert!(
			hash.chars().all(|c| c.is_ascii_hexdigit()),
			"Hash should contain only hex characters"
		);

		let empty_token = "";
		let empty_hash = hash_auth_token(empty_token);
		assert!(!empty_hash.is_empty(), "Even empty token should produce a valid hash");
		assert_ne!(hash, empty_hash, "Empty token should produce different hash than non-empty");

		let test_token = "test";
		let test_hash = hash_auth_token(test_token);
		let expected_hash = blake3::hash(b"test").to_string();
		assert_eq!(test_hash, expected_hash, "Hash should match direct Blake3 computation");
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_validation_specific.sql"
	))]
	async fn test_get_valid_token_with_valid_token(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test user 1 who has a valid token
		let serial_number =
			SerialNumber::from(BigDecimal::from_str("12345678901234567890").unwrap());
		let result = token_store.get_token_userid(&serial_number).await.unwrap();

		assert!(result.is_some());
		assert_eq!(result.unwrap().token.as_str(), "valid_token_hash_1");
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_validation_specific.sql"
	))]
	async fn test_get_valid_token_with_multiple_tokens_returns_latest(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		let serial_number =
			SerialNumber::from(BigDecimal::from_str("98765432109876543210").unwrap());
		let result = token_store.get_token_userid(&serial_number).await.unwrap();

		assert!(result.is_some());
		assert_eq!(result.unwrap().token.as_str(), "valid_token_hash_2");
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_validation_specific.sql"
	))]
	async fn test_get_valid_token_with_no_cert_returns_none(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test user 3 who has no certificate (so no valid tokens)
		let serial_number =
			SerialNumber::from(BigDecimal::from_str("11111111111111111111").unwrap());
		let result = token_store.get_token_userid(&serial_number).await.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_validation_specific.sql"
	))]
	async fn test_get_valid_token_with_nonexistent_serial_returns_none(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test with a serial number that doesn't exist
		let serial_number =
			SerialNumber::from(BigDecimal::from_str("99999999999999999999").unwrap());
		let result = token_store.get_token_userid(&serial_number).await.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_validation_specific.sql"
	))]
	async fn test_get_valid_token_excludes_expired_tokens(pool: Pool<Postgres>) {
		// Insert a user with only expired tokens (using user 5 to avoid conflict with
		// base fixture)
		sqlx::query!(
			"INSERT INTO actors (uaid, type) VALUES
            ('00000000-0000-0000-0000-000000000005', 'local')"
		)
		.execute(&pool)
		.await
		.unwrap();

		sqlx::query!(
			"INSERT INTO local_actors (uaid, local_name, deactivated, joined, password_hash) VALUES
            ('00000000-0000-0000-0000-000000000005', 'test_user_5', false, NOW(), 'hash')"
		)
		.execute(&pool)
		.await
		.unwrap();

		sqlx::query!(
			"INSERT INTO public_keys (id, uaid, pubkey, algorithm_identifier) VALUES
            (7, '00000000-0000-0000-0000-000000000005', 'test_pubkey_7', 1)"
		)
		.execute(&pool)
		.await
		.unwrap();

		sqlx::query!(
            "INSERT INTO idcsr (
                id, serial_number, uaid, actor_public_key_id, actor_signature,
                session_id, valid_not_before, valid_not_after, extensions, pem_encoded
            ) VALUES
            (7, 22222222222222222222, '00000000-0000-0000-0000-000000000005', 7, 'test_signature_7',
             'test_session_7', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_7', 'test_csr_pem_7')"
        )
        .execute(&pool)
        .await
        .unwrap();

		sqlx::query!(
            "INSERT INTO idcert (
                idcsr_id, issuer_info_id, valid_not_before, valid_not_after,
                home_server_public_key_id, home_server_signature, pem_encoded
            ) VALUES
            (7, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 7, 'test_home_server_sig_7', 'test_cert_pem_7')"
        )
        .execute(&pool)
        .await
        .unwrap();

		// Create an additional certificate and ID-CSR for testing multiple expired
		// tokens
		sqlx::query!(
			"INSERT INTO public_keys (id, uaid, pubkey, algorithm_identifier) VALUES
            (9, '00000000-0000-0000-0000-000000000005', 'test_pubkey_9', 1)"
		)
		.execute(&pool)
		.await
		.unwrap();

		sqlx::query!(
            "INSERT INTO idcsr (
                id, serial_number, uaid, actor_public_key_id, actor_signature,
                session_id, valid_not_before, valid_not_after, extensions, pem_encoded
            ) VALUES
            (9, 22222222222222222223, '00000000-0000-0000-0000-000000000005', 9, 'test_signature_9',
             'test_session_9', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_9', 'test_csr_pem_9')"
        )
        .execute(&pool)
        .await
        .unwrap();

		sqlx::query!(
            "INSERT INTO idcert (
                idcsr_id, issuer_info_id, valid_not_before, valid_not_after,
                home_server_public_key_id, home_server_signature, pem_encoded
            ) VALUES
            (9, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 9, 'test_home_server_sig_9', 'test_cert_pem_9')"
        )
        .execute(&pool)
        .await
        .unwrap();

		// Insert only expired tokens
		sqlx::query!(
            "INSERT INTO user_tokens (token_hash, cert_id, uaid, valid_not_after) VALUES
            ('expired_token_hash_7_1', 7, '00000000-0000-0000-0000-000000000005', NOW() - INTERVAL '2 hours'),
            ('expired_token_hash_9_1', 9, '00000000-0000-0000-0000-000000000005', NOW() - INTERVAL '1 hour')"
        )
        .execute(&pool)
        .await
        .unwrap();

		let db = Database { pool };
		let token_store = TokenStore::new(db);
		let serial_number =
			SerialNumber::from(BigDecimal::from_str("22222222222222222222").unwrap());
		let result = token_store.get_token_userid(&serial_number).await.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_validation_specific.sql"
	))]
	async fn test_get_valid_token_with_null_expiration(pool: Pool<Postgres>) {
		// Insert a user with a token that has NULL valid_not_after (using user 6 to
		// avoid conflict)
		sqlx::query!(
			"INSERT INTO actors (uaid, type) VALUES
            ('00000000-0000-0000-0000-000000000006', 'local')"
		)
		.execute(&pool)
		.await
		.unwrap();

		sqlx::query!(
			"INSERT INTO local_actors (uaid, local_name, deactivated, joined, password_hash) VALUES
            ('00000000-0000-0000-0000-000000000006', 'test_user_6', false, NOW(), 'hash')"
		)
		.execute(&pool)
		.await
		.unwrap();

		sqlx::query!(
			"INSERT INTO public_keys (id, uaid, pubkey, algorithm_identifier) VALUES
            (8, '00000000-0000-0000-0000-000000000006', 'test_pubkey_8', 1)"
		)
		.execute(&pool)
		.await
		.unwrap();

		sqlx::query!(
            "INSERT INTO idcsr (
                id, serial_number, uaid, actor_public_key_id, actor_signature,
                session_id, valid_not_before, valid_not_after, extensions, pem_encoded
            ) VALUES
            (8, 33333333333333333333, '00000000-0000-0000-0000-000000000006', 8, 'test_signature_8',
             'test_session_8', NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 'test_extensions_8', 'test_csr_pem_8')"
        )
        .execute(&pool)
        .await
        .unwrap();

		sqlx::query!(
            "INSERT INTO idcert (
                idcsr_id, issuer_info_id, valid_not_before, valid_not_after,
                home_server_public_key_id, home_server_signature, pem_encoded
            ) VALUES
            (8, 1, NOW() - INTERVAL '1 day', NOW() + INTERVAL '1 day', 8, 'test_home_server_sig_8', 'test_cert_pem_8')"
        )
        .execute(&pool)
        .await
        .unwrap();

		// Insert a token with NULL valid_not_after (should be treated as never
		// expiring)
		sqlx::query!(
			"INSERT INTO user_tokens (token_hash, cert_id, uaid, valid_not_after) VALUES
            ('never_expires_token_hash', 8, '00000000-0000-0000-0000-000000000006', NULL)"
		)
		.execute(&pool)
		.await
		.unwrap();

		let db = Database { pool };
		let token_store = TokenStore::new(db);
		let serial_number =
			SerialNumber::from(BigDecimal::from_str("33333333333333333333").unwrap());
		let result = token_store.get_token_userid(&serial_number).await.unwrap();

		assert!(result.is_some());
		assert_eq!(result.unwrap().token.as_str(), "never_expires_token_hash");
	}

	// Tests for get_token_serial_number method
	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_serial_lookup_specific.sql"
	))]
	async fn test_get_token_serial_number_valid_token_returns_correct_serial(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test with valid token hash for user 1
		let result = token_store.get_token_serial_number("token_hash_user_1_a").await.unwrap();

		assert!(result.is_some());
		assert_eq!(
			result.unwrap().as_bigdecimal(),
			&BigDecimal::from_str("12345678901234567890").unwrap()
		);
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_serial_lookup_specific.sql"
	))]
	async fn test_get_token_serial_number_multiple_tokens_same_user(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		let result_a = token_store.get_token_serial_number("token_hash_user_1_a").await.unwrap();
		let result_b = token_store.get_token_serial_number("token_hash_user_1_b").await.unwrap();

		assert!(result_a.is_some());
		assert!(result_b.is_some());

		let serial_a = result_a.as_ref().unwrap();
		let serial_b = result_b.as_ref().unwrap();

		// Different tokens for the same user should have different serial numbers
		// since each token corresponds to a different certificate/ID-CSR
		assert_ne!(serial_a, serial_b);
		assert_eq!(
			serial_a.as_bigdecimal(),
			&BigDecimal::from_str("12345678901234567890").unwrap()
		);
		assert_eq!(
			serial_b.as_bigdecimal(),
			&BigDecimal::from_str("12345678901234567891").unwrap()
		);
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_serial_lookup_specific.sql"
	))]
	async fn test_get_token_serial_number_different_users_different_serials(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test with valid token hashes for different users
		let result_user_1 =
			token_store.get_token_serial_number("token_hash_user_1_a").await.unwrap();
		let result_user_2 =
			token_store.get_token_serial_number("token_hash_user_2_a").await.unwrap();
		let result_user_4 =
			token_store.get_token_serial_number("token_hash_user_4_a").await.unwrap();

		assert!(result_user_1.is_some());
		assert!(result_user_2.is_some());
		assert!(result_user_4.is_some());

		let serial_1 = result_user_1.as_ref().unwrap();
		let serial_2 = result_user_2.as_ref().unwrap();
		let serial_4 = result_user_4.as_ref().unwrap();

		// All should be different
		assert_ne!(serial_1, serial_2);
		assert_ne!(serial_1, serial_4);
		assert_ne!(serial_2, serial_4);

		// Check specific values
		assert_eq!(
			serial_1.as_bigdecimal(),
			&BigDecimal::from_str("12345678901234567890").unwrap()
		);
		assert_eq!(
			serial_2.as_bigdecimal(),
			&BigDecimal::from_str("98765432109876543210").unwrap()
		);
		assert_eq!(
			serial_4.as_bigdecimal(),
			&BigDecimal::from_str("55555555555555555555").unwrap()
		);
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_serial_lookup_specific.sql"
	))]
	async fn test_get_token_serial_number_nonexistent_token_returns_none(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test with token hash that doesn't exist
		let result = token_store.get_token_serial_number("nonexistent_token_hash").await.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_serial_lookup_specific.sql"
	))]
	async fn test_get_token_serial_number_expired_token_still_returns_serial(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		let result =
			token_store.get_token_serial_number("expired_token_hash_user_4").await.unwrap();

		assert!(result.is_some());
		assert_eq!(
			result.unwrap().as_bigdecimal(),
			&BigDecimal::from_str("55555555555555555556").unwrap()
		);
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_serial_lookup_specific.sql"
	))]
	async fn test_get_token_serial_number_empty_token_hash_returns_none(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test with empty token hash
		let result = token_store.get_token_serial_number("").await.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures(
		"../../fixtures/tokens_base_fixture.sql",
		"../../fixtures/token_serial_lookup_specific.sql"
	))]
	async fn test_get_token_serial_number_case_sensitive(pool: Pool<Postgres>) {
		let db = Database { pool };
		let token_store = TokenStore::new(db);

		// Test case sensitivity - should be case sensitive
		let result_lower =
			token_store.get_token_serial_number("token_hash_user_1_a").await.unwrap();
		let result_upper =
			token_store.get_token_serial_number("TOKEN_HASH_USER_1_A").await.unwrap();

		assert!(result_lower.is_some());
		assert!(result_upper.is_none()); // Should not match due to case sensitivity
	}
}

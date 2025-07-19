use log::error;
use polyproto::{der::Encode, key::PublicKey, signature::Signature};
use sqlx::{query, types::Uuid};

use crate::{
    database::{AlgorithmIdentifier, Database},
    errors::{
        ALGORITHM_IDENTIFER_TO_DER_ERROR_MESSAGE, CONTAINS_UNKNOWN_CRYPTO_ALGOS_ERROR_MESSAGE,
        Context, Errcode, Error,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public keys of actors, cached actors and home servers.
pub(crate) struct PublicKeyInfo {
    id: i64,
    pub(crate) uaid: Option<Uuid>,
    pub(crate) pubkey: String,
    pub(crate) algorithm_identifier: i32,
}

impl PublicKeyInfo {
    /// Read-only access to the inner ID field, referencing the ID column in the
    /// database table.
    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    /// Tries to find an entry or entries from the `public_keys` table
    /// matching the given parameter(s). The more parameters given, the more
    /// narrowed down the set of results.
    ///
    /// If all given parameters evaluate to `None`, this function has a fast
    /// path returning an `Ok(Vec::new())`.
    ///
    /// ## Errors
    ///
    /// The function will error, if
    ///
    /// - The database or database connection is broken
    pub(crate) async fn get_by(
        db: &Database,
        uaid: Option<Uuid>,
        pubkey: Option<String>,
        algorithm_identifier: Option<i32>,
        id: Option<i32>,
    ) -> Result<Vec<Self>, Error> {
        if uaid.is_none() && pubkey.is_none() && algorithm_identifier.is_none() && id.is_none() {
            return Ok(Vec::new());
        }
        let record = query!(
            r#"
            SELECT id, uaid, pubkey, algorithm_identifier
            FROM public_keys
            WHERE
                ($1::int IS NULL OR id = $1)
                AND ($2::uuid IS NULL OR uaid = $2)
                AND ($3::text IS NULL OR pubkey = $3)
                AND ($4::int IS NULL OR algorithm_identifier = $4)
        "#,
            id,
            uaid,
            pubkey,
            algorithm_identifier
        )
        .fetch_all(&db.pool)
        .await?;
        Ok(record
            .into_iter()
            .map(|row| PublicKeyInfo {
                id: row.id,
                uaid: row.uaid,
                pubkey: row.pubkey,
                algorithm_identifier: row.algorithm_identifier,
            })
            .collect())
    }

    /// Insert a public key into the `public_keys` table.
    ///
    /// This function extracts algorithm information from the provided public
    /// key, verifies that the cryptographic algorithm is supported by the
    /// server, and inserts the public key information into the database.
    ///
    /// ## Parameters
    ///
    /// - `db` - Database connection reference
    /// - `public_key` - The public key to insert
    /// - `uaid` - Optional user actor ID to associate with the public key
    ///
    /// ## Returns
    ///
    /// Returns the created [PublicKeyInfo] instance on success.
    ///
    /// ## Errors
    ///
    /// The function will error if:
    ///
    /// - The public key uses an unsupported cryptographic algorithm
    /// - The public key already exists in the database
    /// - The associated user does not exist (when UAID is provided)
    /// - Database connection or operation fails
    pub(crate) async fn insert<S: Signature, P: PublicKey<S>>(
        db: &Database,
        public_key: &P,
        uaid: Option<Uuid>,
    ) -> Result<Self, Error> {
        let public_key_algo = public_key.algorithm_identifier();
        let public_key_info = hex::encode(
            public_key.public_key_info().public_key_bitstring.to_der().map_err(|e| {
                error!("{ALGORITHM_IDENTIFER_TO_DER_ERROR_MESSAGE}: {e}");
                Error::new_internal_error(None)
            })?,
        );
        let Some(algorithm_identifiers_row) =
            AlgorithmIdentifier::get_by_algorithm_identifier(db, &public_key_algo).await?
        else {
            error!("Public Key {CONTAINS_UNKNOWN_CRYPTO_ALGOS_ERROR_MESSAGE}");
            return Err(Error::new_internal_error(None));
        };
        let result = query!(
            r#"
            INSERT INTO public_keys (uaid, pubkey, algorithm_identifier)
            VALUES ($1, $2, $3)
            RETURNING id
        "#,
            uaid,
            public_key_info,
            algorithm_identifiers_row.id()
        )
        .fetch_optional(&db.pool)
        .await?;
        // Actually not fully sure of the semantics here: If there is a duplicate, will
        // this throw an error, or will it just return None?
        match result {
            Some(record) => Ok(Self {
                id: record.id,
                uaid,
                pubkey: public_key_info,
                algorithm_identifier: algorithm_identifiers_row.id(),
            }),
            None => Err(Error::new(
                Errcode::IllegalInput,
                Some(Context::new(
                    None,
                    None,
                    None,
                    Some(
                        "Either this public key already has been stored, or the requested user does not exist",
                    ),
                )),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::{Pool, Postgres};

    use super::*;
    use crate::crypto::ed25519::{DigitalPublicKey, DigitalSignature, generate_keypair};

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_get_by_empty_parameters(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = PublicKeyInfo::get_by(&db, None, None, None, None).await.unwrap();

        assert!(result.is_empty(), "Expected empty result when all parameters are None");
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_get_by_id(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = PublicKeyInfo::get_by(&db, None, None, None, Some(1)).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), 1);
        assert_eq!(
            result[0].uaid,
            Some(Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap())
        );
        assert_eq!(result[0].pubkey, "test_pubkey_1");
        assert_eq!(result[0].algorithm_identifier, 1);
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_get_by_uaid(pool: Pool<Postgres>) {
        let db = Database { pool };
        let test_uaid = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let result = PublicKeyInfo::get_by(&db, Some(test_uaid), None, None, None).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].uaid, Some(test_uaid));
        assert_eq!(result[0].pubkey, "test_pubkey_2");
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_get_by_pubkey(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result =
            PublicKeyInfo::get_by(&db, None, Some("test_pubkey_3".to_string()), None, None)
                .await
                .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pubkey, "test_pubkey_3");
        assert_eq!(
            result[0].uaid,
            Some(Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap())
        );
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_get_by_algorithm_identifier(pool: Pool<Postgres>) {
        let db = Database { pool };

        let result = PublicKeyInfo::get_by(&db, None, None, Some(1), None).await.unwrap();

        // Should find all public keys with algorithm_identifier = 1 (RSA)
        assert!(result.len() >= 6); // Based on the fixture data
        for key_info in &result {
            assert_eq!(key_info.algorithm_identifier, 1);
        }
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_get_by_multiple_parameters(pool: Pool<Postgres>) {
        let db = Database { pool };
        let test_uaid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let result =
            PublicKeyInfo::get_by(&db, Some(test_uaid), None, Some(1), None).await.unwrap();

        // Should find public keys for user 1 with algorithm_identifier = 1
        assert_eq!(result.len(), 2); // User 1 has 2 keys in the fixture
        for key_info in &result {
            assert_eq!(key_info.uaid, Some(test_uaid));
            assert_eq!(key_info.algorithm_identifier, 1);
        }
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_get_by_nonexistent_data(pool: Pool<Postgres>) {
        let db = Database { pool };
        let nonexistent_uaid = Uuid::from_str("99999999-9999-9999-9999-999999999999").unwrap();

        let result =
            PublicKeyInfo::get_by(&db, Some(nonexistent_uaid), None, None, None).await.unwrap();

        assert!(result.is_empty(), "Expected empty result for nonexistent UAID");
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_insert_new_key_with_uaid(pool: Pool<Postgres>) {
        let db = Database { pool };
        let (_private_key, public_key) = generate_keypair();
        let test_uaid = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let result = PublicKeyInfo::insert::<DigitalSignature, DigitalPublicKey>(
            &db,
            &public_key,
            Some(test_uaid),
        )
        .await;

        // This should fail because Ed25519 is not in the base fixture (only RSA and EC)
        assert!(result.is_err(), "Expected error because Ed25519 algorithm is not in the fixture");
    }

    #[sqlx::test(fixtures("../../fixtures/tokens_base_fixture.sql"))]
    async fn test_insert_new_key_without_uaid(pool: Pool<Postgres>) {
        let db = Database { pool };
        let (_private_key, public_key) = generate_keypair();

        let result =
            PublicKeyInfo::insert::<DigitalSignature, DigitalPublicKey>(&db, &public_key, None)
                .await;

        // This should fail because Ed25519 is not in the base fixture
        assert!(result.is_err(), "Expected error because Ed25519 algorithm is not in the fixture");
    }

    #[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
    async fn test_insert_ed25519_key_success(pool: Pool<Postgres>) {
        let db = Database { pool };
        let (_private_key, public_key) = generate_keypair();
        let test_uaid = Uuid::from_str("00000000-0000-0000-0000-000000000010").unwrap();

        let result = PublicKeyInfo::insert::<DigitalSignature, DigitalPublicKey>(
            &db,
            &public_key,
            Some(test_uaid),
        )
        .await;

        match result {
            Ok(key_info) => {
                assert_eq!(key_info.uaid, Some(test_uaid));
                assert_eq!(key_info.algorithm_identifier, 3); // Ed25519 algorithm ID from idcert fixture
                assert!(key_info.id() > 0, "Expected positive ID for inserted key");
            }
            Err(e) => {
                panic!("Expected successful insertion with Ed25519 key, but got error: {e:?}");
            }
        }
    }

    #[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
    async fn test_insert_duplicate_key_error(pool: Pool<Postgres>) {
        let db = Database { pool };
        let (_private_key, public_key) = generate_keypair();
        let test_uaid = Uuid::from_str("00000000-0000-0000-0000-000000000010").unwrap();

        // Insert the key once
        let first_result = PublicKeyInfo::insert::<DigitalSignature, DigitalPublicKey>(
            &db,
            &public_key,
            Some(test_uaid),
        )
        .await;
        assert!(first_result.is_ok(), "First insertion should succeed");

        // Try to insert the same key again
        let second_result = PublicKeyInfo::insert::<DigitalSignature, DigitalPublicKey>(
            &db,
            &public_key,
            Some(test_uaid),
        )
        .await;
        assert!(second_result.is_err(), "Second insertion should fail due to duplicate");
    }

    #[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
    async fn test_insert_with_nonexistent_uaid(pool: Pool<Postgres>) {
        let db = Database { pool };
        let (_private_key, public_key) = generate_keypair();
        let nonexistent_uaid = Uuid::from_str("99999999-9999-9999-9999-999999999999").unwrap();

        let result = PublicKeyInfo::insert::<DigitalSignature, DigitalPublicKey>(
            &db,
            &public_key,
            Some(nonexistent_uaid),
        )
        .await;

        assert!(result.is_err(), "Expected error when inserting with nonexistent UAID");
    }

    #[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
    async fn test_get_by_after_insert(pool: Pool<Postgres>) {
        let db = Database { pool };
        let (_private_key, public_key) = generate_keypair();
        let test_uaid = Uuid::from_str("00000000-0000-0000-0000-000000000011").unwrap();

        // Insert a new key
        let inserted_key = PublicKeyInfo::insert::<DigitalSignature, DigitalPublicKey>(
            &db,
            &public_key,
            Some(test_uaid),
        )
        .await
        .unwrap();

        // Retrieve it using get_by
        let retrieved_keys =
            PublicKeyInfo::get_by(&db, None, None, None, Some(inserted_key.id() as i32))
                .await
                .unwrap();

        assert_eq!(retrieved_keys.len(), 1);
        let retrieved_key = &retrieved_keys[0];
        assert_eq!(retrieved_key.id(), inserted_key.id());
        assert_eq!(retrieved_key.uaid, inserted_key.uaid);
        assert_eq!(retrieved_key.pubkey, inserted_key.pubkey);
        assert_eq!(retrieved_key.algorithm_identifier, inserted_key.algorithm_identifier);
    }
}

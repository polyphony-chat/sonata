use chrono::NaiveDateTime;
use log::error;
use polyproto::{
	certs::{PublicKeyInfo, idcert::IdCert},
	der::Encode,
	key::PublicKey,
	signature::Signature,
	types::DomainName,
};
use sqlx::query;

use crate::{
	database::{AlgorithmIdentifier, Database, Issuer},
	errors::{ALGORITHM_IDENTIFER_TO_DER_ERROR_MESSAGE, Context, Error},
};

pub(crate) struct HomeServerCert;

impl HomeServerCert {
	/// Try to get a [HomeServerCert] from the database, filtered by the
	/// [DomainName] and a [NaiveDateTime] timestamp, at which the certificate
	/// must be valid.
	pub(crate) async fn get_idcert_by<S: Signature, P: PublicKey<S>>(
		db: &Database,
		issuer_domain_name: &DomainName,
		timestamp: &NaiveDateTime,
	) -> Result<Option<IdCert<S, P>>, Error> {
		let issuer_components =
			issuer_domain_name.to_string().split('.').map(|s| s.to_owned()).collect::<Vec<_>>();
		let Some(idcert_table_record) = query!(
			r#"
        WITH issuer AS (
            SELECT id
            FROM issuers
            WHERE domain_components = $1
        )
        SELECT idcert.pem_encoded, idcert.home_server_public_key_id
        FROM idcert
        JOIN issuer i ON idcert.issuer_info_id = i.id
        WHERE (
            $2 >= valid_not_before AND $2 <= valid_not_after
        )
    "#,
			issuer_components.as_slice(),
			timestamp
		)
		.fetch_optional(&db.pool)
		.await?
		else {
			return Ok(None);
		};

		let pem_encoded_pubkey_info = query!(
			r#"
        SELECT pubkey
        FROM public_keys
        WHERE id = $1
    "#,
			idcert_table_record.home_server_public_key_id
		)
		.fetch_one(&db.pool)
		.await?;
		IdCert::from_pem(
			&idcert_table_record.pem_encoded,
			polyproto::certs::Target::HomeServer,
			timestamp.and_utc().timestamp() as u64,
			&P::try_from_public_key_info(
				PublicKeyInfo::from_pem(&pem_encoded_pubkey_info.pubkey).map_err(|e| {
					error!("Error parsing public key info: {e}");
					Error::new_internal_error(None)
				})?,
			)
			.map_err(|e| {
				error!("Error creating public key from public key info: {e}");
				Error::new_internal_error(None)
			})?,
		)
		.map_err(|e| {
			error!("Error parsing home server certificate: {e}");
			Error::new_internal_error(None)
		})
		.map(Some)
	}

	///
	pub(crate) async fn insert_idcert_unchecked<S: Signature, P: PublicKey<S>>(
		db: &Database,
		cert: IdCert<S, P>,
	) -> Result<(), Error> {
		let oid_signature_algo = S::algorithm_identifier().oid;
		let params_signature_algo = match S::algorithm_identifier().parameters {
			Some(params) => params.to_der().map_err(|e| {
				error!("{ALGORITHM_IDENTIFER_TO_DER_ERROR_MESSAGE} {e}");
				Error::new_internal_error(None)
			})?,
			None => Vec::new(),
		};
		let Some(algorithm_identifier) = AlgorithmIdentifier::get_by_query(
			db,
			None,
			None,
			Some(&oid_signature_algo),
			&params_signature_algo,
		)
		.await?
		.first() else {
			return Err(Error::new(
				crate::errors::Errcode::IllegalInput,
				Some(Context::new(
					None,
					None,
					None,
					Some("ID-Cert contains cryptographic algorithms not supported by this server"),
				)),
			));
		};
		#[allow(clippy::expect_used)]
		// This event should never happen and, as far as I am aware, cannot be triggered by any
		// user. As such, I see it ok to unwrap here.
		let issuer = Issuer::get_own(db).await?.expect(
			"The issuer entry for this sonata instance should have been added to the database on startup!",
		);
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use chrono::{NaiveDate, Utc};
	use sqlx::{Pool, Postgres, query};

	use super::*;
	use crate::crypto::ed25519::{DigitalPublicKey, DigitalSignature, generate_keypair};

	/// Helper function to update fixture with real ED25519 keys and mock
	/// certificates
	// TODO: use real certs
	async fn setup_real_keys_mock_certs(pool: &Pool<Postgres>) {
		// Generate keypairs functionally and convert to PEM
		let public_key_updates: Vec<(i64, String)> = (0..6)
			.map(|_| generate_keypair().1) // Take only public key
			.map(|pubkey| pubkey.public_key_info().to_pem(polyproto::der::pem::LineEnding::LF))
			.collect::<Result<Vec<_>, _>>()
			.expect("Failed to encode public keys to PEM")
			.into_iter()
			.zip([100i64, 101, 102, 103, 200, 201])
			.map(|(pem, id)| (id, pem))
			.collect();

		// Apply updates to database
		for (id, pem) in public_key_updates {
			query!("UPDATE public_keys SET pubkey = $1 WHERE id = $2", pem, id)
				.execute(pool)
				.await
				.unwrap_or_else(|_| panic!("Failed to update public key {}", id));
		}

		// Generate mock certificates functionally
		let mock_cert_data = [
			"MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAy8Dbv8prpJ/0kKhlGeJY",
			"MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1gKdWHX6Zv8ZLNqXwC7D",
			"MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA2hLdVGY7Wx9YMNpXzE8G",
		];

		let certificate_updates: Vec<(i64, String)> = mock_cert_data
			.iter()
			.map(|&data| {
				format!("-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----", data)
			})
			.zip([100i64, 101, 102])
			.map(|(pem, id)| (id, pem))
			.collect();

		// Apply certificate updates to database
		for (id, pem) in certificate_updates {
			query!("UPDATE idcert SET pem_encoded = $1 WHERE idcsr_id = $2", pem, id)
				.execute(pool)
				.await
				.unwrap_or_else(|_| panic!("Failed to update certificate {}", id));
		}
	}

	#[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
	async fn test_get_idcert_by_nonexistent_domain(pool: Pool<Postgres>) {
		setup_real_keys_mock_certs(&pool).await;
		let db = Database { pool };

		let domain = DomainName::new("nonexistent.com").unwrap();
		let timestamp = Utc::now().naive_utc();

		let result = HomeServerCert::get_idcert_by::<DigitalSignature, DigitalPublicKey>(
			&db, &domain, &timestamp,
		)
		.await
		.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
	async fn test_get_idcert_by_expired_certificate(pool: Pool<Postgres>) {
		setup_real_keys_mock_certs(&pool).await;
		let db = Database { pool };

		// expired.net has a certificate that's already expired
		let domain = DomainName::new("expired.net").unwrap();
		let timestamp = Utc::now().naive_utc();

		let result = HomeServerCert::get_idcert_by::<DigitalSignature, DigitalPublicKey>(
			&db, &domain, &timestamp,
		)
		.await
		.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
	async fn test_get_idcert_by_future_timestamp(pool: Pool<Postgres>) {
		setup_real_keys_mock_certs(&pool).await;
		let db = Database { pool };

		let domain = DomainName::new("example.com").unwrap();
		// Set timestamp far in the future, beyond certificate validity
		let future_timestamp =
			NaiveDate::from_ymd_opt(2030, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();

		let result = HomeServerCert::get_idcert_by::<DigitalSignature, DigitalPublicKey>(
			&db,
			&domain,
			&future_timestamp,
		)
		.await
		.unwrap();

		assert!(result.is_none());
	}

	#[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
	async fn test_get_idcert_by_past_timestamp(pool: Pool<Postgres>) {
		setup_real_keys_mock_certs(&pool).await;
		let db = Database { pool };

		let domain = DomainName::new("example.com").unwrap();
		// Set timestamp in the past, before certificate validity
		let past_timestamp =
			NaiveDate::from_ymd_opt(2020, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();

		let result = HomeServerCert::get_idcert_by::<DigitalSignature, DigitalPublicKey>(
			&db,
			&domain,
			&past_timestamp,
		)
		.await
		.unwrap();

		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_get_idcert_by_domain_case_sensitivity() {
		// Test domain validation behavior
		let domain_exact = DomainName::new("example.com");
		assert!(domain_exact.is_ok(), "Lowercase domain should be valid");

		// Test that uppercase domain names are invalid per domain validation rules
		let domain_upper_result = DomainName::new("EXAMPLE.COM");
		assert!(
			domain_upper_result.is_err(),
			"Uppercase domain names should be rejected by DomainName validation"
		);

		println!("Domain case sensitivity validation works correctly");
	}

	#[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
	async fn test_get_idcert_by_multiple_domains(pool: Pool<Postgres>) {
		setup_real_keys_mock_certs(&pool).await;
		let db = Database { pool };

		let timestamp = Utc::now().naive_utc();

		// Test example.com
		let domain1 = DomainName::new("example.com").unwrap();
		let result1 = HomeServerCert::get_idcert_by::<DigitalSignature, DigitalPublicKey>(
			&db, &domain1, &timestamp,
		)
		.await;

		// Test test.org
		let domain2 = DomainName::new("test.org").unwrap();
		let result2 = HomeServerCert::get_idcert_by::<DigitalSignature, DigitalPublicKey>(
			&db, &domain2, &timestamp,
		)
		.await;

		// Both should find database records but fail on certificate parsing for now
		// TODO
		assert!(result1.is_err());
		assert!(result2.is_err());
	}

	#[sqlx::test(fixtures("../../fixtures/idcert_integration_tests.sql"))]
	async fn test_get_idcert_by_database_edge_cases(pool: Pool<Postgres>) {
		setup_real_keys_mock_certs(&pool).await;
		let db = Database { pool };

		// Test with subdomain that doesn't exist
		let subdomain = DomainName::new("sub.example.com").unwrap();
		let timestamp = Utc::now().naive_utc();

		let result_subdomain = HomeServerCert::get_idcert_by::<DigitalSignature, DigitalPublicKey>(
			&db, &subdomain, &timestamp,
		)
		.await
		.unwrap();

		assert!(result_subdomain.is_none());

		// Test with empty domain components (this should fail domain creation)
		let empty_domain_result = DomainName::new("");
		assert!(empty_domain_result.is_err());
	}

	#[tokio::test]
	async fn test_real_ed25519_key_generation_and_pem_encoding() {
		let (_private_key, public_key) = generate_keypair();

		// Test PEM encoding/decoding pipeline functionally
		let pem_data = public_key
			.public_key_info()
			.to_pem(polyproto::der::pem::LineEnding::LF)
			.expect("Failed to encode public key to PEM");

		// Verify PEM structure functionally
		[
			("-----BEGIN PUBLIC KEY-----", pem_data.starts_with("-----BEGIN PUBLIC KEY-----")),
			("-----END PUBLIC KEY-----\n", pem_data.ends_with("-----END PUBLIC KEY-----\n")),
		]
		.iter()
		.for_each(|(expected, valid)| {
			assert!(*valid, "PEM structure validation failed for: {}", expected);
		});

		// Test round-trip: PEM -> PublicKeyInfo -> DigitalPublicKey -> bytes
		let original_bytes = public_key.key.to_bytes();
		let reconstructed_bytes = PublicKeyInfo::from_pem(&pem_data)
			.and_then(|info| DigitalPublicKey::try_from_public_key_info(info))
			.map(|key| key.key.to_bytes())
			.expect("Failed to reconstruct key from PEM");

		assert_eq!(original_bytes, reconstructed_bytes, "Round-trip key conversion failed");
	}
}

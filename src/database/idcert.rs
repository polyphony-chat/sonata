use chrono::NaiveDateTime;
use log::error;
use polyproto::{
	certs::{PublicKeyInfo, idcert::IdCert},
	key::PublicKey,
	signature::Signature,
	types::DomainName,
};
use sqlx::query;

use crate::{database::Database, errors::Error};

pub(crate) struct HomeServerCert;

impl HomeServerCert {
	/// TODO documentme
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
}

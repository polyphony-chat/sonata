use chrono::NaiveDateTime;
use polyproto::{certs::idcert::IdCert, types::DomainName};

use crate::{
	config::SonataConfig,
	crypto::ed25519::{DigitalPublicKey, DigitalSignature},
	database::Database,
	errors::{Context, Error},
};

pub(crate) struct Issuer {
	pub(crate) id: i64,
	pub(crate) domain_components: DomainName,
	pub(crate) id_cert: IdCert<DigitalSignature, DigitalPublicKey>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Issuer {
	/// Create (insert) the issuer entry for this sonata instance. Returns
	/// `None`, if there is already a valid issuer entry for the given point in
	/// time.
	pub(crate) async fn create_own(
		db: &Database,
		time: NaiveDateTime,
	) -> Result<Option<Self>, Error> {
		let config_domain = &SonataConfig::get_or_panic().general.server_domain;
		let domain_name = DomainName::new(&config_domain).map_err(|e| {
			Error::new(
				crate::errors::Errcode::IllegalInput,
				Some(Context::new(None, None, None, Some(&e.to_string()))),
			)
		})?;
		todo!()
	}
}

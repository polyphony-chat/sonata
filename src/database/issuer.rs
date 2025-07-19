use log::error;
use polyproto::types::DomainName;
use sqlx::query;

use crate::{
	config::SonataConfig,
	database::Database,
	errors::{Context, Error},
};

pub(crate) struct Issuer {
	pub(crate) id: i64,
	pub(crate) domain_components: DomainName,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Issuer {
	/// Create (insert) the issuer entry for this sonata instance.
	pub(crate) async fn create_own(db: &Database) -> Result<Option<Self>, Error> {
		let config_domain = &SonataConfig::get_or_panic().general.server_domain;
		let domain_name = DomainName::new(config_domain).map_err(|e| {
			Error::new(
				crate::errors::Errcode::IllegalInput,
				Some(Context::new(None, None, None, Some(&e.to_string()))),
			)
		})?;
		let domain_name_separated =
			domain_name.to_string().split('.').map(|s| s.to_owned()).collect::<Vec<_>>();
		let record = query!(
			r#"
			INSERT INTO issuers (domain_components)
			VALUES ($1)
			ON CONFLICT (domain_components) DO NOTHING
			RETURNING id, domain_components
		"#,
			&domain_name_separated
		)
		.fetch_optional(&db.pool)
		.await?;
		match record {
			Some(row) => Ok(Some(Issuer {
				id: row.id,
				domain_components: match DomainName::new(&row.domain_components.join(".")) {
					Err(e) => {
						error!("Error: Invalid DomainName stored in issuers table: {e}");
						return Err(Error::new_internal_error(None));
					}
					Ok(dn) => dn,
				},
			})),
			None => Ok(None),
		}
	}
}

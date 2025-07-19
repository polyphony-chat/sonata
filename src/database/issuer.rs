use log::error;
use polyproto::types::DomainName;
use sqlx::query;

use crate::{
	config::SonataConfig,
	database::Database,
	errors::{Context, Error},
};

/// Represents an issuer row in the database table with the same name.
pub(crate) struct Issuer {
	/// ID of this issuer
	id: i64,
	/// The [DomainName] of this issuer.
	pub(crate) domain_components: DomainName,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Issuer {
	/// Read-only access to the inner ID field, referencing the ID column in the
	/// database table.
	pub(crate) fn id(&self) -> i64 {
		self.id
	}

	/// Convert a [DomainName] into a `Vec<String>`
	fn domain_name_to_vec_string(domain_name: DomainName) -> Vec<String> {
		domain_name.to_string().split('.').map(|s| s.to_owned()).collect::<Vec<_>>()
	}

	/// Convert a `Vec<String>` to a [DomainName]
	fn vec_string_to_domain_name(strs: Vec<String>) -> Result<DomainName, Box<Error>> {
		match DomainName::new(&strs.join(".")) {
			Err(e) => {
				error!("Error: Invalid DomainName stored in issuers table: {e}");
				Err(Error::new_internal_error(None).into())
			}
			Ok(dn) => Ok(dn),
		}
	}

	/// Convert a `str` to a [DomainName]
	fn str_to_domain_name(string: &str) -> Result<DomainName, Box<Error>> {
		DomainName::new(string).map_err(|e| {
			Error::new(
				crate::errors::Errcode::IllegalInput,
				Some(Context::new(None, None, None, Some(&e.to_string()))),
			)
			.into()
		})
	}

	/// Create (insert) the issuer entry for this sonata instance.
	pub(crate) async fn create_own(db: &Database) -> Result<Option<Self>, Error> {
		let config_domain = &SonataConfig::get_or_panic().general.server_domain;
		let domain_name = Self::str_to_domain_name(config_domain).map_err(|e| *e)?;
		let domain_name_separated = Self::domain_name_to_vec_string(domain_name);
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
				domain_components: Self::vec_string_to_domain_name(row.domain_components)
					.map_err(|e| *e)?,
			})),
			None => Ok(None),
		}
	}

	/// Get the issuer entryfor this sonata instance from the database. Returns
	/// `Ok(None)`, if the item does not exist.
	pub(crate) async fn get_own(db: &Database) -> Result<Option<Self>, Error> {
		let domain_name =
			Self::str_to_domain_name(&SonataConfig::get_or_panic().general.server_domain)
				.map_err(|e| *e)?;
		let record = query!(
			r#"
			SELECT id, domain_components
			FROM issuers
			WHERE domain_components = $1
		"#,
			&Self::domain_name_to_vec_string(domain_name)
		)
		.fetch_optional(&db.pool)
		.await?;
		Ok(match record {
			Some(row) => Some(Self {
				id: row.id,
				domain_components: Self::vec_string_to_domain_name(row.domain_components)
					.map_err(|e| *e)?,
			}),
			None => None,
		})
	}
}

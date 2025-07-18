use polyproto::spki::ObjectIdentifier;
use sqlx::query;

use crate::{
	database::Database,
	errors::{Context, Error},
};

pub(crate) struct AlgorithmIdentifier {
	id: i32,
	pub(crate) algorithm_identifier: ObjectIdentifier,
	pub(crate) common_name: Option<String>,
	pub(crate) parameters: Option<String>,
}

impl AlgorithmIdentifier {
	/// Tries to find an entry or entries from the `algorithm_identifiers` table
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
	/// - Any entry in the set of elements queried from the database contains
	///   text in the `algorithm_identifier` column, which is not in valid,
	///   dot-delimited OID string form.
	pub(crate) async fn get_by(
		db: &Database,
		id: Option<i32>,
		common_name: Option<&str>,
		algorithm_identifier: Option<&ObjectIdentifier>,
		parameters: Option<&str>,
	) -> Result<Vec<Self>, Error> {
		if common_name.is_none()
			&& id.is_none()
			&& algorithm_identifier.is_none()
			&& parameters.is_none()
		{
			return Ok(Vec::new());
		}
		let record = query!(
			r#"
            SELECT id, algorithm_identifier, common_name, parameters
            FROM algorithm_identifiers
            WHERE
                ($1::int IS NULL OR id = $1)
                AND ($2::text IS NULL OR algorithm_identifier = $2)
                AND ($3::text IS NULL OR common_name = $3)
                AND ($4::text IS NULL OR parameters = $4)
            "#,
			id,
			algorithm_identifier.map(|a| a.to_string()),
			common_name,
			parameters,
		)
		.fetch_all(&db.pool)
		.await?;
		let algorithm_identifiers_mapped = record
			.into_iter()
			.flat_map(|r| {
				Ok(AlgorithmIdentifier {
					id: r.id,
					algorithm_identifier: match ObjectIdentifier::new(&r.algorithm_identifier) {
						Ok(oid) => oid,
						Err(e) => {
							return Err(Error::new(
								crate::errors::Errcode::Internal,
								Some(Context::new(
									None,
									Some(&r.algorithm_identifier),
									None,
									Some(&format!(
										"Found invalid algorithm_identifier in table algorithm_identifiers: {e}"
									)),
								)),
							));
						}
					},
					common_name: r.common_name,
					parameters: r.parameters,
				})
			})
			.collect::<Vec<_>>();
		Ok(algorithm_identifiers_mapped)
	}

	/// Tries to insert a new row into the `algorithm_identifiers` table.
	///
	/// ## Errors
	///
	/// This function will return the following errors in the following cases:
	///
	/// ### `Error` with `Errcode::Duplicate`
	///
	/// Returned, when one or more of the `UNIQUE` constraints of the database
	/// schema have been violated. The database is not modified at all in this
	/// case.
	///
	/// ### `Error` with `Errcode::IllegalInput`
	///
	/// Returned, when the text in the `algorithm_identifier` column is not in
	/// valid, dot-delimited OID string form.
	pub(crate) async fn try_insert(
		db: &Database,
		algorithm_identifier: &ObjectIdentifier,
		common_name: Option<&str>,
		parameters: Option<&str>,
	) -> Result<Self, Error> {
		let record = query!(
			r#"
        INSERT INTO algorithm_identifiers (algorithm_identifier, common_name, parameters)
        VALUES ($1, $2::text, $3::text)
        ON CONFLICT DO NOTHING RETURNING id, algorithm_identifier, common_name, parameters
        "#,
			algorithm_identifier.to_string(),
			common_name,
			parameters
		)
		.fetch_optional(&db.pool)
		.await?;

		match record {
			Some(row) => Ok(AlgorithmIdentifier {
				id: row.id,
				algorithm_identifier: match ObjectIdentifier::new(&row.algorithm_identifier) {
					Ok(oid) => oid,
					Err(e) => {
						return Err(Error::new_internal_error(Some(&format!(
							"Found invalid algorithm_identifier in table algorithm_identifiers: {e}"
						))));
					}
				},
				common_name: row.common_name,
				parameters: row.parameters,
			}),
			None => Err(Error::new_duplicate_error(Some(
				"The provided algorithm identifier is already present in the database",
			))),
		}
	}
}

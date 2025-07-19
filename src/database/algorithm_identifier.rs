use log::error;
use polyproto::{
    der::Encode,
    spki::{AlgorithmIdentifierOwned, ObjectIdentifier},
};
use sqlx::query;

use crate::{
    database::Database,
    errors::{ALGORITHM_IDENTIFER_TO_DER_ERROR_MESSAGE, Error},
};

pub(crate) struct AlgorithmIdentifier {
    id: i32,
    pub(crate) algorithm_identifier: ObjectIdentifier,
    pub(crate) common_name: Option<String>,
    pub(crate) parameters_der_encoded: Option<Vec<u8>>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl AlgorithmIdentifier {
    /// Read-only access to the inner ID field, referencing the ID column in the
    /// database table.
    pub(crate) fn id(&self) -> i32 {
        self.id
    }

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
    pub(crate) async fn get_by_query(
        db: &Database,
        id: Option<i32>,
        common_name: Option<&str>,
        algorithm_identifier: Option<&ObjectIdentifier>,
        parameters_der_encoded: &[u8],
    ) -> Result<Vec<Self>, Error> {
        if common_name.is_none()
            && id.is_none()
            && algorithm_identifier.is_none()
            && parameters_der_encoded.is_empty()
        {
            return Ok(Vec::new());
        }
        let parameters_der_encoded_reformatted =
            parameters_der_encoded.into_iter().map(|num| *num as i16).collect::<Vec<_>>();
        let parameters_for_query = if parameters_der_encoded_reformatted.is_empty() {
            None
        } else {
            Some(parameters_der_encoded_reformatted.as_slice())
        };
        let record = query!(
            r#"
            SELECT id, algorithm_identifier, common_name, parameters_der_encoded
            FROM algorithm_identifiers
            WHERE
                ($1::int IS NULL OR id = $1)
                AND ($2::text IS NULL OR algorithm_identifier = $2)
                AND ($3::text IS NULL OR common_name = $3)
                AND ($4::smallint [] IS NULL OR parameters_der_encoded = $4 OR (parameters_der_encoded IS NULL AND $4::smallint [] = '{}'))
            "#,
            id,
            algorithm_identifier.map(|a| a.to_string()),
            common_name,
            parameters_for_query,
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
							error!(
								"Found invalid algorithm_identifier in table algorithm_identifiers: {e}"
							);
							return Err(Error::new_internal_error(None));
						}
					},
					common_name: r.common_name,
					parameters_der_encoded: r
						.parameters_der_encoded
						.map(|v| v.into_iter().map(|num| num as u8).collect::<Vec<_>>()),
				})
			})
			.collect::<Vec<_>>();
        Ok(algorithm_identifiers_mapped)
    }

    /// Tries to get the row entry [AlgorithmIdentifier] matching an
    /// [AlgorithmIdentifierOwned].
    ///
    /// ## Errors
    ///
    /// The function will error, if
    ///
    /// - The database or database connection is broken
    /// - Any entry in the set of elements queried from the database contains
    ///   text in the `algorithm_identifier` column, which is not in valid,
    ///   dot-delimited OID string form.
    pub(crate) async fn get_by_algorithm_identifier(
        db: &Database,
        algorithm_identifier: &AlgorithmIdentifierOwned,
    ) -> Result<Option<Self>, Error> {
        let parameters_der_encoded = algorithm_identifier.parameters.to_der().map_err(|e| {
            error!("{ALGORITHM_IDENTIFER_TO_DER_ERROR_MESSAGE}: {e}");
            Error::new_internal_error(None)
        })?;
        let oid = algorithm_identifier.oid;
        let mut result =
            Self::get_by_query(db, None, None, Some(&oid), &parameters_der_encoded).await?;
        Ok(if !result.is_empty() { Some(result.swap_remove(0)) } else { None })
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
        parameters: &[u8],
    ) -> Result<Self, Error> {
        let parameters_i16 = parameters.into_iter().map(|num| *num as i16).collect::<Vec<_>>();
        let record = query!(
			r#"
        INSERT INTO algorithm_identifiers (algorithm_identifier, common_name, parameters_der_encoded)
        VALUES ($1, $2::text, $3::smallint [])
        ON CONFLICT DO NOTHING RETURNING id, algorithm_identifier, common_name, parameters_der_encoded
        "#,
			algorithm_identifier.to_string(),
			common_name,
			&parameters_i16
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
                parameters_der_encoded: row
                    .parameters_der_encoded
                    .map(|inner| inner.into_iter().map(|num| num as u8).collect::<Vec<_>>()),
            }),
            None => Err(Error::new_duplicate_error(Some(
                "The provided algorithm identifier is already present in the database",
            ))),
        }
    }
}

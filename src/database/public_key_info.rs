use polyproto::{key::PublicKey, signature::Signature};
use sqlx::{query, types::Uuid};

use crate::{
    database::{AlgorithmIdentifier, Database},
    errors::Error,
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

    pub(crate) async fn insert<S: Signature, P: PublicKey<S>>(
        db: &Database,
        public_key: &P,
    ) -> Result<Self, Error> {
        let public_key_algo = public_key.algorithm_identifier();
        let Some(algorithm_identifiers_row) =
            AlgorithmIdentifier::get_by_algorithm_identifier(db, &public_key_algo).await?
        else {};
        todo!()
    }
}

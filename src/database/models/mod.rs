// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::NaiveDateTime;
use sqlx::{query, types::Uuid};

use crate::{database::Database, errors::SonataDbError};

#[derive(sqlx::FromRow, sqlx::Type)]
pub struct PemEncoded(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorType {
	Local,
	Foreign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Actor {
	pub unique_actor_identifier: Uuid,
	r#type: ActorType,
}

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
/// Actors from this home server.
pub struct LocalActor {
	/// The unique actor identifer. Does not change, even if the `local_name`
	/// changes.
	pub unique_actor_identifier: Uuid,
	/// The "local name" part of the actor. See the polyproto specification for
	/// more information.
	pub local_name: String,
	/// Whether this actors' account is currently deactivated.
	pub is_deactivated: bool,
	/// Timestamp from when the actor has first registered on the server, or
	/// when this account has been created.
	pub joined_at_timestamp: chrono::NaiveDateTime,
}

impl LocalActor {
	/// Tries to find an actor from the [Database] where `local_name` is equal
	/// to `name`, returning `None`, if such an actor does not exist.
	///
	/// ## Errors
	///
	/// Will error on Database connection issues and on other errors with the
	/// database, all of which are not in scope for this function to handle.
	pub async fn by_local_name(
		db: &Database,
		name: &str,
	) -> Result<Option<LocalActor>, SonataDbError> {
		Ok(query!(
			"
            SELECT uaid, local_name, deactivated, joined
            FROM local_actors
            WHERE local_name = $1
            LIMIT 1",
			name
		)
		.fetch_optional(&db.pool)
		.await?
		.map(|record| LocalActor {
			unique_actor_identifier: record.uaid,
			local_name: record.local_name,
			is_deactivated: record.deactivated,
			joined_at_timestamp: record.joined,
		}))
	}
}

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct AlgorithmIdentifier {
	pub id: i32,
	pub algorithm_identifier_oid: String,
	pub common_name: Option<String>,
	pub parameters: Option<String>,
}

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct PublicKey {
	pub id: i64,
	pub belongs_to_actor_identifier: Uuid,
	pub public_key: String,
	pub algorithm_identifier_id: i32,
}

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct Subjects {
	pub actor_unique_identifier: Uuid,
	pub domain_components: Vec<String>,
	pub subject_x509_pem: PemEncoded,
}

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct Issuers {
	pub id: i64,
	pub domain_components: Vec<String>,
	pub issuer_x509_pem: PemEncoded,
}

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct IdCsr {
	pub internal_serial_number: Uuid,
	pub for_actor_uaid: Uuid,
	pub actor_public_key_id: i64,
	pub actor_signature: String,
	pub session_id: String, // TODO make this serialnumba
	pub valid_not_before: NaiveDateTime,
	pub valid_not_after: NaiveDateTime,
	pub extensions: String,
	pub pem_encoded: String,
}

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct Invite {
	pub invite_link_owner: Option<Uuid>,
	pub usages_current: i32,
	pub usages_maximum: i32,
	pub invite_code: String,
	pub invalid: bool,
}

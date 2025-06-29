// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use chrono::NaiveDateTime;
use sqlx::types::Uuid;

#[derive(sqlx::FromRow, sqlx::Type)]
pub struct PemEncoded(String);

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct Actor {
    pub unique_actor_identifier: Uuid,
    pub local_name: String,
    pub is_deactivated: bool,
    pub joined_at_timestamp: chrono::NaiveDateTime,
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

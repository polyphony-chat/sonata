use chrono::NaiveDateTime;
use polyproto::certs::SessionId;
use polyproto::types::x509_cert::SerialNumber;

#[derive(sqlx::FromRow, sqlx::Type)]
/// Represents a **Universally Unique Identifier** (UUID). From Wikipedia:
///
/// > A Universally Unique Identifier (UUID) is a 128-bit label used to uniquely identify objects in
/// > computer systems. The term Globally Unique Identifier (GUID) is also used, mostly in Microsoft
/// > systems. When generated according to the standard methods, UUIDs are, for practical purposes,
/// > unique. Their uniqueness does not depend on a central registration authority or coordination
/// > between the parties generating them, unlike most other numbering schemes. While the probability
/// > that a UUID will be duplicated is not zero, it is generally considered close enough to zero to
/// > be negligible.
pub struct Uuid(String);

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
    pub session_id: SerialNumber,
    pub valid_not_before: NaiveDateTime,
    pub valid_not_after: NaiveDateTime,
    pub extensions: String,
    pub pem_encoded: String,
}

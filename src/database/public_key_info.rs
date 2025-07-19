use sqlx::types::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublicKeyInfo {
	id: i64,
	pub(crate) uaid: Option<Uuid>,
	pub(crate) pubkey: String,
	pub(crate) algorithm_identifier: i32,
}

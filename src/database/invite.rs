use sqlx::types::Uuid;

#[derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow)]
pub struct Invite {
	pub invite_link_owner: Option<Uuid>,
	pub usages_current: i32,
	pub usages_maximum: i32,
	pub invite_code: String,
	pub invalid: bool,
}

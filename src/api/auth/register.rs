use argon2::{
	Argon2,
	password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use poem::{
	IntoResponse, Response, handler,
	http::StatusCode,
	web::{Data, Json},
};
use serde_json::json;

use crate::{
	api::models::{NISTPasswordRequirements, PasswordRequirements, RegisterSchema},
	database::{Database, LocalActor, tokens::TokenStore},
	errors::{Context, Errcode, Error},
};

#[cfg_attr(coverage_nightly, coverage(off))]
#[handler]
pub async fn register(
	Json(payload): Json<RegisterSchema>,
	Data(db): Data<&Database>,
	Data(token_store): Data<&TokenStore>,
) -> Result<impl IntoResponse, Error> {
	// TODO: Check if registration is currently allowed
	// TODO: Check for tos_consent
	// TODO: Check if registration is currently in invite-only mode
	if LocalActor::by_local_name(db, &payload.local_name).await?.is_some() {
		return Err(Error::new(
			Errcode::Duplicate,
			Some(Context::new(Some("local_name"), Some(&payload.local_name), None)),
		));
	}
	let password = NISTPasswordRequirements::verify_requirements(&payload.password)?;
	let salt = SaltString::generate(&mut OsRng);
	let argon2 = Argon2::default();
	let password_hash = argon2
		.hash_password(password.as_bytes(), &salt)
		.map_err(|_| Error::new(Errcode::Internal, None))?;
	// TODO: Check if registration is currently in whitelist mode
	let new_user =
		LocalActor::create(db, &payload.local_name, password_hash.serialize().as_str()).await?;
	let token_hash =
		token_store.generate_upsert_token(&new_user.unique_actor_identifier, None).await?;
	Ok(Response::builder()
		.status(StatusCode::CREATED)
		.body(json!({"token": token_hash}).to_string()))
}

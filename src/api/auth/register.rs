use argon2::{
	Argon2,
	password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use poem::{
	IntoResponse, handler,
	http::StatusCode,
	web::{Data, Json},
};

use crate::{
	api::models::{NISTPasswordRequirements, PasswordRequirements, RegisterSchema},
	database::{Database, LocalActor, tokens::TokenStore},
	errors::{Context, Errcode, Error, SonataApiError},
};

#[cfg_attr(coverage_nightly, coverage(off))]
#[handler]
pub async fn register(
	Json(payload): Json<RegisterSchema>,
	Data(db): Data<&Database>,
	Data(token_store): Data<&TokenStore>,
) -> Result<impl IntoResponse, SonataApiError> {
	// TODO: Check if registration is currently allowed
	// TODO: Check if registration is currently in invite-only mode
	if LocalActor::by_local_name(db, &payload.local_name).await?.is_some() {
		return Err(SonataApiError::Error(Error::new(
			Errcode::Duplicate,
			Some(Context::new(Some("local_name"), Some(&payload.local_name), None)),
		)));
	}
	let password = NISTPasswordRequirements::verify_requirements(&payload.password)?;
	let salt = SaltString::generate(&mut OsRng);
	let argon2 = Argon2::default();
	let password_hash = argon2
		.hash_password(password.as_bytes(), &salt)
		.map_err(|_| Error::new(Errcode::Internal, None).into_api_error())?;
	// TODO: Check if registration is currently in whitelist mode
	// TODO: Store user etc. in DB
	Ok(poem::error::Error::from_status(StatusCode::NOT_IMPLEMENTED).into_response())
}

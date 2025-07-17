use argon2::{Argon2, PasswordHash, PasswordVerifier};
use log::error;
use poem::{
	IntoResponse, Response, handler,
	http::StatusCode,
	web::{Data, Json},
};
use serde_json::json;

use crate::{
	MAX_PERMITTED_PASSWORD_LEN,
	api::auth::models::LoginSchema,
	database::{Database, LocalActor, tokens::TokenStore},
	errors::{Context, Errcode, Error},
};

#[handler]
pub(super) async fn login(
	Json(payload): Json<LoginSchema>,
	Data(db): Data<&Database>,
	Data(token_store): Data<&TokenStore>,
) -> Result<impl IntoResponse, Error> {
	if payload.password.len() > MAX_PERMITTED_PASSWORD_LEN {
		return Err(Error::new(
			Errcode::IllegalInput,
			Some(Context::new(
				Some("password"),
				Some(&format!("{} characters", payload.password.len())),
				Some(&format!("Not more than {MAX_PERMITTED_PASSWORD_LEN} characters")),
				None,
			)),
		));
	}
	let local_actor = match LocalActor::by_local_name(db, &payload.local_name).await? {
		Some(actor) => actor,
		None => return Err(Error::invalid_login()),
	};
	let actor_password_hashstring =
		match LocalActor::get_password_hash(db, &payload.local_name).await? {
			Some(hash_string) => hash_string,
			None => {
				return Err(Error::invalid_login());
			}
		};
	let actor_password_hash = PasswordHash::new(&actor_password_hashstring).map_err(|e| {
		error!(
			"Password hash for user {} is not in PHC string format? Got error: {e}",
			payload.password
		);
		Error::new(Errcode::Internal, None)
	})?;
	Argon2::default()
		.verify_password(payload.password.as_bytes(), &actor_password_hash)
		.map_err(|_| Error::invalid_login())?;
	let token =
		token_store.generate_upsert_token(&local_actor.unique_actor_identifier, None).await?;
	Ok(Response::builder().status(StatusCode::OK).body(json!({"token": token}).to_string()))
}

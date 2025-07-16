use poem::{
	IntoResponse,
	web::{Data, Json},
};

use crate::{
	api::models::LoginSchema,
	database::{Database, tokens::TokenStore},
	errors::SonataApiError,
};

pub async fn login(
	Json(payload): Json<LoginSchema>,
	Data(db): Data<&Database>,
	Data(token_store): Data<&TokenStore>,
) -> Result<impl IntoResponse, SonataApiError> {
	if payload.password.len() > 128 {
		return;
	}
	Ok(String::new())
}

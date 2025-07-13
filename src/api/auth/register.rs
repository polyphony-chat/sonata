use poem::{
	IntoResponse, handler,
	http::StatusCode,
	web::{Data, Json},
};

use crate::{
	api::models::RegisterSchema,
	database::{Database, tokens::TokenStore},
	errors::SonataApiError,
};

#[handler]
pub async fn register(
	Json(payload): Json<RegisterSchema>,
	Data(db): Data<&Database>,
	Data(token_store): Data<&TokenStore>,
) -> Result<impl IntoResponse, SonataApiError> {
	Ok(poem::error::Error::from_status(StatusCode::NOT_IMPLEMENTED).into_response())
}

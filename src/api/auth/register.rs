use poem::http::StatusCode;
use poem::web::Json;
use poem::{IntoResponse, handler};

use crate::api::models::RegisterSchema;
use crate::errors::SonataApiError;

#[handler]
pub async fn register(
    Json(payload): Json<RegisterSchema>,
) -> Result<impl IntoResponse, SonataApiError> {
    Ok(poem::error::Error::from_status(StatusCode::NOT_IMPLEMENTED).into_response())
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Display;

use derive_more::Display;
use poem::{IntoResponse, Response, error::ResponseError, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Generic error type.
pub(crate) type StdError = Box<dyn std::error::Error + Sync + Send + 'static>;
/// Generic result type.
pub(crate) type StdResult<T> = Result<T, StdError>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
	pub code: Errcode,
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub context: Option<Context>,
}

impl Error {
	pub fn into_response_body(self) -> String {
		json!(self).to_string()
	}

	pub fn new(code: Errcode, context: Option<Context>) -> Self {
		Self { code, message: code.message(), context }
	}
}

#[derive(Debug, Clone, Copy, Display, Serialize, Deserialize)]
pub enum Errcode {
	#[display("P2_CORE_INTERNAL")]
	Internal,
}

impl Errcode {
	pub fn message(&self) -> String {
		match self {
			Errcode::Internal => {
				"An internal error has occurred and this request cannot be processed further"
					.to_string()
			}
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
	#[serde(skip_serializing_if = "String::is_empty")]
	pub field_name: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub found: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	pub expected: String,
}

impl Context {
	pub fn new(field_name: Option<&str>, found: Option<&str>, expected: Option<&str>) -> Self {
		Self {
			field_name: field_name.map(String::from).unwrap_or_default(),
			found: found.map(String::from).unwrap_or_default(),
			expected: expected.map(String::from).unwrap_or_default(),
		}
	}
}

#[derive(Debug, thiserror::Error)]
/// Error type for errors that concern the HTTP API. Implements
/// [poem::error::ResponseError].
pub(crate) enum SonataApiError {
	#[error(transparent)]
	/// Generic error variant, supporting any type implementing
	/// [std::error::Error].
	StdError(StdError),
	/// A DB-related error.
	#[error(transparent)]
	DbError(SonataDbError),
}

#[derive(Debug, thiserror::Error)]
/// Error type for errors that concern interactions with the Database.
/// Implements [poem::error::ResponseError].
pub(crate) enum SonataGatewayError {
	#[error(transparent)]
	/// Generic error variant, supporting any type implementing
	/// [std::error::Error].
	StdError(StdError),
}

#[derive(Debug, thiserror::Error)]
/// Error type for errors that concern the Database or Database connection.
pub(crate) enum SonataDbError {
	#[error(transparent)]
	/// Generic error variant, supporting any type implementing
	/// [std::error::Error].
	StdError(StdError),
	#[error(transparent)]
	/// An [sqlx::Error]
	Sqlx(#[from] sqlx::Error),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ResponseError for SonataApiError {
	fn status(&self) -> poem::http::StatusCode {
		match self {
			SonataApiError::StdError(_) => StatusCode::INTERNAL_SERVER_ERROR,
			SonataApiError::DbError(sonata_db_error) => sonata_db_error.status(),
		}
	}
}

impl IntoResponse for SonataApiError {
	fn into_response(self) -> Response {
		Response::builder().status(self.status()).body(match self {
			SonataApiError::StdError(_) => Error::new(Errcode::Internal, None).into_response_body(),
			SonataApiError::DbError(_) => Error::new(Errcode::Internal, None).into_response_body(),
		})
	}
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ResponseError for SonataDbError {
	fn status(&self) -> poem::http::StatusCode {
		match self {
			SonataDbError::StdError(_) => StatusCode::INTERNAL_SERVER_ERROR,
			SonataDbError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

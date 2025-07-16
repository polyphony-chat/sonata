// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Display;

use derive_more::{Display, FromStr};
use poem::{IntoResponse, Response, error::ResponseError, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// Generic error type.
pub(crate) type StdError = Box<dyn std::error::Error + Sync + Send + 'static>;
/// Generic result type.
pub(crate) type StdResult<T> = Result<T, StdError>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A polyproto core Error, with an [Errcode], an error message and optional
/// error [Context].
///
/// Convenience struct to make poem-compatible and unified error returning
/// easier
pub struct Error {
	/// The error code [Errcode], giving a rough idea of what went wrong
	pub code: Errcode,
	/// An error message, providing some further information about the category
	/// of error encountered.
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	/// Optional error context.
	///
	/// ## Example
	///
	/// If a password has to be at least 8
	/// characters long, but the user only supplied 6, the context field could
	/// tell the user that the field `password` in their request is wrong, and
	/// supply a very fine-grained error message, telling the user that they
	/// only supplied 6 characters, while 8 were required.
	pub context: Option<Context>,
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.to_response_body())
	}
}

impl Error {
	/// Performs the conversion of [Self] into a [SonataApiError].
	pub fn into_api_error(self) -> SonataApiError {
		SonataApiError::Error(self)
	}
	/// Performs the conversion of a shared reference to [Self] into JSON,
	/// formatted as a string.
	pub fn to_response_body(&self) -> String {
		json!(self).to_string()
	}

	/// Creates [Self].
	pub fn new(code: Errcode, context: Option<Context>) -> Self {
		Self { code, message: code.message(), context }
	}
}

#[derive(Debug, Clone, Copy, Display, PartialEq, FromStr, DeserializeFromStr, SerializeDisplay)]
/// Standardized polyproto core error codes, giving a rough idea of what went
/// wrong.
pub enum Errcode {
	#[display("P2_CORE_INTERNAL")]
	/// An internal error occurred.
	Internal,
	#[display("P2_CORE_UNAUTHORIZED")]
	/// Unauthorized
	Unauthorized,
	#[display("P2_CORE_DUPLICATE")]
	/// The resource already exists, and the context does not allow for
	/// duplicate resources
	Duplicate,
	#[display("P2_CORE_ILLEGAL_INPUT")]
	/// One or many parts of the given input did not succeed validation against
	/// context-specific criteria
	IllegalInput,
}

impl Errcode {
	/// Get an error message, describing what the error code itself means.
	pub fn message(&self) -> String {
		match self {
    Errcode::Internal => {
				"An internal error has occurred and this request cannot be processed further"
					.to_owned()
			}
    Errcode::Unauthorized => {
				"This action requires authorization, proof of which was not granted".to_owned()
			}
    Errcode::Duplicate => {
				"Creation of the resource is not possible, as it already exists".to_owned()
			}
    Errcode::IllegalInput => "The overall input is well-formed, but one or more of the input fields fail validation criteria".to_owned(),
            }
	}
}

impl ResponseError for Errcode {
	fn status(&self) -> StatusCode {
		match self {
			Errcode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
			Errcode::Unauthorized => StatusCode::UNAUTHORIZED,
			Errcode::Duplicate => StatusCode::CONFLICT,
			Errcode::IllegalInput => StatusCode::BAD_REQUEST,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Optional error context.
///
/// ## Example
///
/// If a password has to be at least 8
/// characters long, but the user only supplied 6, the context field could
/// tell the user that the `field_name` "`password`" in their request is wrong,
/// and supply a very fine-grained error message, telling the user that they
/// only supplied 6 characters, while 8 were required.
pub struct Context {
	#[serde(skip_serializing_if = "String::is_empty")]
	/// The name of the request body field which caused the error
	pub field_name: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	/// The value that was found to be fault inside the `field_name`
	pub found: String,
	#[serde(skip_serializing_if = "String::is_empty")]
	/// The value that was expected
	pub expected: String,
}

impl Context {
	/// Creates [Self].
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
/// TODO rewrite documentation
pub(crate) enum SonataApiError {
	/// Generic error variant, supporting any type implementing
	/// [std::error::Error].
	#[error("{0}")]
	StdError(Error),
	/// A DB-related error.
	#[error("{0}")]
	DbError(Error),
	#[error("{0}")]
	Error(Error),
}

impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		log::error!("{value}");
		Self::new(Errcode::Internal, None)
	}
}

impl From<StdError> for Error {
	fn from(value: StdError) -> Self {
		log::error!("{value}");
		Self::new(Errcode::Internal, None)
	}
}

impl From<Error> for SonataApiError {
	fn from(value: Error) -> Self {
		Self::Error(value)
	}
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

impl ResponseError for SonataApiError {
	fn status(&self) -> poem::http::StatusCode {
		match self {
			SonataApiError::StdError(_) => StatusCode::INTERNAL_SERVER_ERROR,
			SonataApiError::Error(error) | SonataApiError::DbError(error) => error.code.status(),
		}
	}
}

impl IntoResponse for SonataApiError {
	fn into_response(self) -> Response {
		Response::builder().status(self.status()).body(match self {
			SonataApiError::StdError(_) => Error::new(Errcode::Internal, None).to_response_body(),
			SonataApiError::DbError(_) => Error::new(Errcode::Internal, None).to_response_body(),
			SonataApiError::Error(error) => error.to_response_body(),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_error_serialization() {
		let context = Context::new(Some("field"), Some("value"), Some("expected"));
		let error = Error {
			code: Errcode::IllegalInput,
			message: "Test message".to_string(),
			context: Some(context),
		};

		let serialized = serde_json::to_string(&error).unwrap();
		let deserialized: Error = serde_json::from_str(&serialized).unwrap();

		assert_eq!(deserialized.code, error.code);
		assert_eq!(deserialized.message, error.message);
		assert!(deserialized.context.is_some());
		let ctx = deserialized.context.unwrap();
		assert_eq!(ctx.field_name, "field");
		assert_eq!(ctx.found, "value");
		assert_eq!(ctx.expected, "expected");
	}
}

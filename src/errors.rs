// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use poem::{IntoResponse, Response, error::ResponseError, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// Generic error type.
pub(crate) type StdError = Box<dyn std::error::Error + Sync + Send + 'static>;
/// Generic result type.
pub(crate) type StdResult<T> = Result<T, StdError>;

/// Error message to log when converting an [AlgorithmIdentifierOwner] to DER
/// encoding fails.
pub(crate) const ALGORITHM_IDENTIFER_TO_DER_ERROR_MESSAGE: &str =
    "Error encoding signature algorithm parameters to DER:";
/// Error message to log when an insertion into the database fails, beacuse user
/// data contained unsupported cryptographic primitives.
///
/// ## Formatting
///
/// in `format!()`, this variable should be used as follows:
///
/// ```
/// use crate::errors::CONTAINS_UNKNOWN_CRYPTO_ALGOS_ERROR_MESSAGE;
///
/// fn function() {
///     format!("ID-Cert {CONTAINS_UNKNOWN_CRYPTO_ALGOS_ERROR_MESSAGE}");
///     format!("Public Key {CONTAINS_UNKNOWN_CRYPTO_ALGOS_ERROR_MESSAGE}");
/// }
/// ```
pub(crate) const CONTAINS_UNKNOWN_CRYPTO_ALGOS_ERROR_MESSAGE: &str =
    "contains cryptographic algorithms not supported by this server";

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

impl IntoResponse for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn into_response(self) -> Response {
        Response::builder()
            .content_type("application/json")
            .status(self.code.status())
            .body(self.to_json())
    }
}

impl ResponseError for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn status(&self) -> StatusCode {
        self.code.status()
    }
}

impl From<sqlx::Error> for Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(value: sqlx::Error) -> Self {
        log::error!("{value}");
        Error::new(Errcode::Internal, None)
    }
}

impl From<Error> for poem::Error {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(value: Error) -> Self {
        poem::Error::from_response(value.into_response())
    }
}

/// Error message for a wrong username or password.
pub const ERROR_WRONG_LOGIN: &str = "The provided login name or password was incorrect.";

impl Error {
    /// Performs the conversion of a shared reference to [Self] into JSON,
    /// formatted as a string.
    #[must_use]
    pub fn to_json(&self) -> String {
        json!(self).to_string()
    }

    /// Creates [Self].
    #[must_use]
    pub fn new(code: Errcode, context: Option<Context>) -> Self {
        Self { code, message: code.message(), context }
    }

    /// Creates a variant of [Self] which indicates to a client, that the
    /// provided combination of login name and password was incorrect, without
    /// telling the client what the concrete issue was.
    ///
    /// This helper method is useful, because inconsistencies between error
    /// messages indicating wrong login credentials could potentially leave an
    /// attacker with information about internal state they are not supposed to
    /// know about.
    #[must_use = "Not returning this variant as a response opens up the possibility of leaking internal state!"]
    pub fn new_invalid_login() -> Self {
        Error::new(
            Errcode::Unauthorized,
            Some(Context::new(None, None, None, Some(ERROR_WRONG_LOGIN))),
        )
    }

    /// Creates a variant of [Self] with an [Errcode] of `Errcode::Internal` and
    /// an optional, given message.
    pub fn new_internal_error(message: Option<&str>) -> Self {
        Self::new(Errcode::Internal, Some(Context::new(None, None, None, message)))
    }

    /// Creates a variant of [Self] with an [Errcode] of `Errcode::Duplicate`
    /// and an optional, given message.
    pub fn new_duplicate_error(message: Option<&str>) -> Self {
        Self::new(Errcode::Duplicate, Some(Context::new(None, None, None, message)))
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    DeserializeFromStr,
    SerializeDisplay,
    strum::Display,
    strum::EnumString,
)]
/// Standardized polyproto core error codes, giving a rough idea of what went
/// wrong.
pub enum Errcode {
    #[strum(serialize = "P2_CORE_INTERNAL")]
    /// An internal error occurred.
    Internal,
    #[strum(serialize = "P2_CORE_UNAUTHORIZED")]
    /// Unauthorized
    Unauthorized,
    #[strum(serialize = "P2_CORE_DUPLICATE")]
    /// The resource already exists, and the context does not allow for
    /// duplicate resources
    Duplicate,
    #[strum(serialize = "P2_CORE_ILLEGAL_INPUT")]
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
    #[cfg_attr(coverage_nightly, coverage(off))]
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
    #[serde(default)]
    /// The name of the request body field which caused the error
    pub field_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    /// The value that was found to be fault inside the `field_name`
    pub found: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    /// The value that was expected
    pub expected: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    /// An optional, additional, human-readable error message
    pub message: String,
}

impl Context {
    /// Creates [Self].
    pub fn new(
        field_name: Option<&str>,
        found: Option<&str>,
        expected: Option<&str>,
        message: Option<&str>,
    ) -> Self {
        Self {
            field_name: field_name.map(String::from).unwrap_or_default(),
            found: found.map(String::from).unwrap_or_default(),
            expected: expected.map(String::from).unwrap_or_default(),
            message: message.map(String::from).unwrap_or_default(),
        }
    }

    /// Creates [Self], with only the `message` field being `Some`.
    pub fn new_message(message: &str) -> Self {
        Self::new(None, None, None, Some(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let context = Context::new(Some("field"), Some("value"), Some("expected"), Some("message"));
        let error = Error::new(Errcode::IllegalInput, Some(context));

        let serialized = serde_json::to_string(&error).unwrap();
        println!("{serialized}");
        let deserialized: Error = serde_json::from_str(&serialized).unwrap();
        println!("{deserialized:#?}");

        assert_eq!(deserialized.code, error.code);
        assert_eq!(deserialized.message, error.message);
        assert!(deserialized.context.is_some());
        let ctx = deserialized.context.unwrap();
        assert_eq!(ctx.field_name, "field");
        assert_eq!(ctx.found, "value");
        assert_eq!(ctx.expected, "expected");
        assert_eq!(ctx.message, "message");
    }

    #[test]
    fn test_error_without_context() {
        let error = Error::new(Errcode::Internal, None);

        assert_eq!(error.code, Errcode::Internal);
        assert_eq!(
            error.message,
            "An internal error has occurred and this request cannot be processed further"
        );
        assert!(error.context.is_none());
    }

    #[test]
    fn test_error_to_json() {
        let context = Context::new(Some("username"), Some("admin"), Some("valid username"), None);
        let error = Error::new(Errcode::Duplicate, Some(context));

        let json = error.to_json();
        assert!(json.contains("P2_CORE_DUPLICATE"));
        assert!(json.contains("username"));
        assert!(json.contains("admin"));
        assert!(json.contains("valid username"));
    }

    #[test]
    fn test_error_new_invalid_login() {
        let error = Error::new_invalid_login();

        assert_eq!(error.code, Errcode::Unauthorized);
        assert_eq!(
            error.message,
            "This action requires authorization, proof of which was not granted"
        );
        assert!(error.context.is_some());
        let ctx = error.context.unwrap();
        assert_eq!(ctx.message, ERROR_WRONG_LOGIN);
    }

    #[test]
    fn test_error_new_internal_error() {
        let error = Error::new_internal_error(Some("Database connection failed"));

        assert_eq!(error.code, Errcode::Internal);
        assert!(error.context.is_some());
        let ctx = error.context.unwrap();
        assert_eq!(ctx.message, "Database connection failed");
    }

    #[test]
    fn test_error_new_duplicate_error() {
        let error = Error::new_duplicate_error(Some("User already exists"));

        assert_eq!(error.code, Errcode::Duplicate);
        assert!(error.context.is_some());
        let ctx = error.context.unwrap();
        assert_eq!(ctx.message, "User already exists");
    }

    #[test]
    fn test_errcode_messages() {
        assert_eq!(
            Errcode::Internal.message(),
            "An internal error has occurred and this request cannot be processed further"
        );
        assert_eq!(
            Errcode::Unauthorized.message(),
            "This action requires authorization, proof of which was not granted"
        );
        assert_eq!(
            Errcode::Duplicate.message(),
            "Creation of the resource is not possible, as it already exists"
        );
        assert_eq!(
            Errcode::IllegalInput.message(),
            "The overall input is well-formed, but one or more of the input fields fail validation criteria"
        );
    }

    #[test]
    fn test_errcode_status_codes() {
        use poem::http::StatusCode;

        assert_eq!(Errcode::Internal.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(Errcode::Unauthorized.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(Errcode::Duplicate.status(), StatusCode::CONFLICT);
        assert_eq!(Errcode::IllegalInput.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_errcode_serialization() {
        let internal = Errcode::Internal;
        let serialized = serde_json::to_string(&internal).unwrap();
        assert_eq!(serialized, "\"P2_CORE_INTERNAL\"");

        let deserialized: Errcode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Errcode::Internal);
    }

    #[test]
    fn test_context_new() {
        let context =
            Context::new(Some("password"), Some("weak"), Some("strong"), Some("Password too weak"));

        assert_eq!(context.field_name, "password");
        assert_eq!(context.found, "weak");
        assert_eq!(context.expected, "strong");
        assert_eq!(context.message, "Password too weak");
    }

    #[test]
    fn test_context_new_with_none_values() {
        let context = Context::new(None, None, None, Some("General error"));

        assert!(context.field_name.is_empty());
        assert!(context.found.is_empty());
        assert!(context.expected.is_empty());
        assert_eq!(context.message, "General error");
    }

    #[test]
    fn test_error_into_response() {
        let error = Error::new(Errcode::IllegalInput, None);
        let response = error.into_response();

        assert_eq!(response.status(), poem::http::StatusCode::BAD_REQUEST);
        assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    }

    #[test]
    fn test_error_from_sqlx_error() {
        use sqlx::Error as SqlxError;

        let sqlx_error = SqlxError::RowNotFound;
        let error: Error = sqlx_error.into();

        assert_eq!(error.code, Errcode::Internal);
        assert!(error.context.is_none());
    }

    #[test]
    fn test_error_into_poem_error() {
        let error = Error::new(Errcode::Unauthorized, None);
        let poem_error: poem::Error = error.into();

        // We can't directly test the poem::Error contents, but we can ensure the
        // conversion works
        assert_eq!(poem_error.status(), poem::http::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_errcode_display() {
        assert_eq!(Errcode::Internal.to_string(), "P2_CORE_INTERNAL");
        assert_eq!(Errcode::Unauthorized.to_string(), "P2_CORE_UNAUTHORIZED");
        assert_eq!(Errcode::Duplicate.to_string(), "P2_CORE_DUPLICATE");
        assert_eq!(Errcode::IllegalInput.to_string(), "P2_CORE_ILLEGAL_INPUT");
    }

    #[test]
    fn test_errcode_from_str() {
        use std::str::FromStr;

        assert_eq!(Errcode::from_str("P2_CORE_INTERNAL").unwrap(), Errcode::Internal);
        assert_eq!(Errcode::from_str("P2_CORE_UNAUTHORIZED").unwrap(), Errcode::Unauthorized);
        assert_eq!(Errcode::from_str("P2_CORE_DUPLICATE").unwrap(), Errcode::Duplicate);
        assert_eq!(Errcode::from_str("P2_CORE_ILLEGAL_INPUT").unwrap(), Errcode::IllegalInput);

        assert!(Errcode::from_str("INVALID_CODE").is_err());
    }
}

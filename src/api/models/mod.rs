use serde::{Deserialize, Serialize};

use crate::errors::{Context, Error};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Information sent to the server by a client, when the client wants to create
/// a new account.
///
/// ## Important Note
///
/// sonata is in an MVP phase. As such, things like this `RegisterSchema` are
/// subject to a lot of change. If you build clients around sonata, expect
/// things to break in future versions.
pub struct RegisterSchema {
	/// Whether the client has agreed to the terms of service offered by the
	/// instance.
	pub tos_consent: bool,
	/// The local name the client would like to choose
	pub local_name: String,
	/// A password for the clients' new account
	pub password: String,
	/// Optional: An invite code, which the client got referred to this instance
	/// with.
	#[serde(default)]
	pub invite: Option<String>,
	/// Key acquired by solving a captcha
	#[serde(default)]
	pub captcha_key: Option<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Information sent to the server by a client, when the client wants to login.
///
/// ## Important Note
///
/// sonata is in an MVP phase. As such, things like this `LoginSchema` are
/// subject to a lot of change. If you build clients around sonata, expect
/// things to break in future versions.
pub struct LoginSchema {
	/// The login username
	pub local_name: String,
	/// Login password
	pub password: String,
	/// Key acquired by solving a captcha
	#[serde(default)]
	pub captcha_key: Option<String>,
}

/// A trait to verify that a password string matches a set of requirements, such
/// as length, composition details, permitted character set, etc.
pub trait PasswordRequirements {
	/// Verify that a password string matches a set of requirements, such
	/// as length, composition details, permitted character set, etc.
	///
	/// Returns a [String] containing the input password, if the verification
	/// has been passed.
	fn verify_requirements(password: &str) -> Result<String, Error>;
}

/// A very basic manifestation of NIST 2024 password security guidelines,
/// stating:
///
/// - All Unicode characters are allowed, including the space (` `) character
/// - Passwords must be at least 8 characters in length and should be at least
///   64 characters in length (this implementation chooses 64 as a limit)
/// - No password composition rules are enforced (Numbers, uppercase, lowercase
///   characters are not enforced)
///
/// ## Warning
///
/// This is not a certified implementation of a NIST standard and does not claim
/// to be one. This structs' purpose is to supply a non-frustrating set of
/// password verification guidelines via the [PasswordRequirements] trait.
pub struct NISTPasswordRequirements;

impl PasswordRequirements for NISTPasswordRequirements {
	fn verify_requirements(password: &str) -> Result<String, Error> {
		let len = password.len();
		if !(8..=64).contains(&len) {
			return Err(Error::new(
				crate::errors::Errcode::IllegalInput,
				Some(Context::new(
					Some("password"),
					Some(&(len.to_string() + " characters")),
					Some("More than 7 and less than 65 characters"),
				)),
			));
		}
		Ok(password.to_owned())
	}
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::str_to_string)]
mod tests {

	use super::*;

	#[test]
	fn test_register_schema_serialization() {
		let schema = RegisterSchema {
			tos_consent: true,
			local_name: "testuser".to_string(),
			password: "testpassword123".to_string(),
			invite: Some("invite123".to_string()),
			captcha_key: Some("captcha".to_string()),
		};

		let serialized = serde_json::to_string(&schema).unwrap();
		let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

		assert_eq!(parsed["tosConsent"], true);
		assert_eq!(parsed["localName"], "testuser");
		assert_eq!(parsed["password"], "testpassword123");
		assert_eq!(parsed["invite"], "invite123");
		assert_eq!(parsed["captchaKey"], "captcha");
	}

	#[test]
	fn test_register_schema_deserialization() {
		let json_str = r#"{"tosConsent":true,"localName":"testuser","password":"testpassword123","invite":"invite123"}"#;
		let schema: RegisterSchema = serde_json::from_str(json_str).unwrap();

		assert!(schema.tos_consent);
		assert_eq!(schema.local_name, "testuser");
		assert_eq!(schema.password, "testpassword123");
		assert_eq!(schema.invite, Some("invite123".to_string()));
	}

	#[test]
	fn test_nist_password_requirements_valid_password() {
		let result = NISTPasswordRequirements::verify_requirements("password123");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), "password123");
	}

	#[test]
	fn test_nist_password_requirements_minimum_length() {
		let result = NISTPasswordRequirements::verify_requirements("12345678");
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), "12345678");
	}

	#[test]
	fn test_nist_password_requirements_maximum_length() {
		let long_password = "a".repeat(64);
		let result = NISTPasswordRequirements::verify_requirements(&long_password);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), long_password);
	}

	#[test]
	fn test_nist_password_requirements_too_short() {
		let result = NISTPasswordRequirements::verify_requirements("1234567");
		assert!(result.is_err());
		let error = result.unwrap_err();
		assert_eq!(error.code, crate::errors::Errcode::IllegalInput);
		assert!(error.context.is_some());
		let context = error.context.unwrap();
		assert_eq!(context.field_name, "password");
		assert_eq!(context.found, "7 characters");
		assert_eq!(context.expected, "More than 7 and less than 65 characters");
	}

	#[test]
	fn test_nist_password_requirements_too_long() {
		let long_password = "a".repeat(65);
		let result = NISTPasswordRequirements::verify_requirements(&long_password);
		assert!(result.is_err());
		let error = result.unwrap_err();
		assert_eq!(error.code, crate::errors::Errcode::IllegalInput);
		assert!(error.context.is_some());
		let context = error.context.unwrap();
		assert_eq!(context.field_name, "password");
		assert_eq!(context.found, "65 characters");
		assert_eq!(context.expected, "More than 7 and less than 65 characters");
	}

	#[test]
	fn test_nist_password_requirements_empty_password() {
		let result = NISTPasswordRequirements::verify_requirements("");
		assert!(result.is_err());
		let error = result.unwrap_err();
		assert_eq!(error.code, crate::errors::Errcode::IllegalInput);
		assert!(error.context.is_some());
		let context = error.context.unwrap();
		assert_eq!(context.field_name, "password");
		assert_eq!(context.found, "0 characters");
		assert_eq!(context.expected, "More than 7 and less than 65 characters");
	}

	#[test]
	fn test_nist_password_requirements_unicode_characters() {
		let unicode_password = "пароль123🔐";
		let result = NISTPasswordRequirements::verify_requirements(unicode_password);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), unicode_password);
	}

	#[test]
	fn test_nist_password_requirements_spaces_allowed() {
		let password_with_spaces = "password with spaces";
		let result = NISTPasswordRequirements::verify_requirements(password_with_spaces);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), password_with_spaces);
	}

	#[test]
	fn test_nist_password_requirements_special_characters() {
		let special_password = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
		let result = NISTPasswordRequirements::verify_requirements(special_password);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), special_password);
	}

	#[test]
	fn test_nist_password_requirements_mixed_case() {
		let mixed_case_password = "AbCdEfGhIjKl";
		let result = NISTPasswordRequirements::verify_requirements(mixed_case_password);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), mixed_case_password);
	}

	#[test]
	fn test_nist_password_requirements_numbers_only() {
		let numbers_only = "12345678";
		let result = NISTPasswordRequirements::verify_requirements(numbers_only);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), numbers_only);
	}

	#[test]
	fn test_nist_password_requirements_letters_only() {
		let letters_only = "abcdefgh";
		let result = NISTPasswordRequirements::verify_requirements(letters_only);
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), letters_only);
	}

	#[test]
	fn test_nist_password_requirements_boundary_values() {
		// Test exactly 8 characters
		let min_valid = "12345678";
		assert!(NISTPasswordRequirements::verify_requirements(min_valid).is_ok());

		// Test exactly 64 characters
		let max_valid = "a".repeat(64);
		assert!(NISTPasswordRequirements::verify_requirements(&max_valid).is_ok());

		// Test exactly 7 characters (should fail)
		let too_short = "1234567";
		assert!(NISTPasswordRequirements::verify_requirements(too_short).is_err());

		// Test exactly 65 characters (should fail)
		let too_long = "a".repeat(65);
		assert!(NISTPasswordRequirements::verify_requirements(&too_long).is_err());
	}
}

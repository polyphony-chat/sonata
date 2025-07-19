use serde::{Deserialize, Serialize};

// TODO: captcha_key for RegisterSchema and LoginSchema

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
	pub invite: Option<String>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Information sent to the server by a client, when the client wants to log
/// into an account.
///
/// ## Important Note
///
/// sonata is in an MVP phase. As such, things like this `LoginSchema` are
/// subject to a lot of change. If you build clients around sonata, expect
/// things to break in future versions.
pub struct LoginSchema {
	/// The name of the account the client wants to login to
	pub local_name: String,
	/// The password of the account the client wants to login to
	pub password: String,
}

#[cfg(test)]
#[allow(
	clippy::unwrap_used,
	clippy::str_to_string,
	clippy::indexing_slicing,
	clippy::bool_assert_comparison
)]
mod test {
	use super::*;
	#[test]
	fn test_register_schema_serialization() {
		let schema = RegisterSchema {
			tos_consent: true,
			local_name: "testuser".to_string(),
			password: "testpassword123".to_string(),
			invite: Some("invite123".to_string()),
		};

		let serialized = serde_json::to_string(&schema).unwrap();
		let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

		assert_eq!(parsed["tosConsent"], true);
		assert_eq!(parsed["localName"], "testuser");
		assert_eq!(parsed["password"], "testpassword123");
		assert_eq!(parsed["invite"], "invite123");
	}

	#[test]
	fn test_register_schema_deserialization() {
		let json_str = r#"{"tosConsent":true,"localName":"testuser","password":"testpassword123","invite":"invite123"}"#;
		let schema: RegisterSchema = serde_json::from_str(json_str).unwrap();

		assert_eq!(schema.tos_consent, true);
		assert_eq!(schema.local_name, "testuser");
		assert_eq!(schema.password, "testpassword123");
		assert_eq!(schema.invite, Some("invite123".to_string()));
	}
}

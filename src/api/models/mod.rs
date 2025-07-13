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
	pub invite: Option<String>,
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

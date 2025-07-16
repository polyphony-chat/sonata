// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::Deref;

use rand::{
	distr::{Alphanumeric, SampleString},
	prelude::ThreadRng,
};
use sqlx::query;

use crate::{StdError, database::Database, errors::Error};

/// Constant used to determine how long auto-generated tokens are supposed to
/// be.
pub const STANDARD_TOKEN_LENGTH: usize = 128;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ApiKey {
	token: String,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Deref for ApiKey {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.token
	}
}

impl ApiKey {
	/// Create a new `ApiKey`. API Keys must be >= 32 and <= 255 characters in
	/// length. If your input string meets these two conditions, you will
	/// receive an `Ok(ApiKey)`.
	pub fn new(token: &str) -> Result<Self, Error> {
		if token.is_empty() {
			return Err(String::from("Token must not be empty").into());
		}
		if token.len() < 32 {
			return Err(String::from("Token must be at least 32 characters in length").into());
		}
		if token.len() > 255 {
			return Err(String::from("Token must not be longer than 255 characters").into());
		}
		Ok(Self { token: token.to_owned() })
	}

	#[cfg_attr(coverage_nightly, coverage(off))]
	/// Getter for the internal token.
	pub fn token(&self) -> &str {
		&self.token
	}

	/// Generates a new, random [ApiKey] which is [STANDARD_TOKEN_LENGTH]
	/// characters in length.
	pub fn new_random(rng: &mut ThreadRng) -> Self {
		Self { token: Alphanumeric.sample_string(rng, STANDARD_TOKEN_LENGTH) }
	}
}

impl std::fmt::Display for ApiKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self)
	}
}

/// Create an [ApiKey] from the given `token`, then insert it into the database.
pub(crate) async fn add_api_key_to_database(
	token: &str,
	database: &Database,
) -> Result<ApiKey, sqlx::Error> {
	let key = ApiKey::new(token).map_err(SonataDbError::StdError)?;
	query!("INSERT INTO api_keys (token) VALUES ($1)", key.token())
		.execute(&database.pool)
		.await
		.map_err(SonataDbError::Sqlx)?;
	Ok(key)
}

#[cfg(test)]
mod test {
	use rand::rng;
	use sqlx::{Pool, Postgres};

	use super::*;

	#[test]
	fn token_length() {
		assert!(ApiKey::new("").is_err());
		assert!(ApiKey::new("token").is_err());
		assert!(ApiKey::new(['c'; 256].iter().collect::<String>().as_str()).is_err());
		assert!(ApiKey::new("transrightsarehumanrights_thefirstpridewasariot").is_ok());
	}

	#[test]
	fn auto_gen_token() {
		assert_eq!(ApiKey::new_random(&mut rng()).len(), STANDARD_TOKEN_LENGTH);
	}

	#[sqlx::test]
	async fn insert_key_into_db(db: Pool<Postgres>) {
		let key = ApiKey::new_random(&mut rng());
		assert!(add_api_key_to_database(key.token(), &Database { pool: db }).await.is_ok());
	}
}

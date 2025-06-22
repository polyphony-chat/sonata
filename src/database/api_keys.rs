use std::ops::Deref;

use rand::distr::{Alphanumeric, SampleString};
use rand::prelude::ThreadRng;
use sqlx::query;

use crate::StdError;
use crate::database::Database;
use crate::errors::SonataDbError;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ApiKey {
    token: String,
}

impl Deref for ApiKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

impl ApiKey {
    /// Create a new `ApiKey`. API Keys must be >= 32 and <= 255 characters in length. If your
    /// input string meets these two conditions, you will receive an `Ok(ApiKey)`.
    pub fn new(token: &str) -> Result<Self, StdError> {
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

    pub fn token(&self) -> &str {
        &self.token
    }

    /// Generates a new, random [ApiKey] which is 128 characters in length.
    pub fn new_random(rng: &mut ThreadRng) -> Self {
        Self { token: Alphanumeric.sample_string(rng, 128) }
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
) -> Result<ApiKey, crate::errors::SonataDbError> {
    let key = ApiKey::new(token).map_err(SonataDbError::StdError)?;
    query!("INSERT INTO api_keys (token) VALUES ($1)", key.token())
        .execute(&database.pool)
        .await
        .map_err(SonataDbError::Sqlx)?;
    Ok(key)
}

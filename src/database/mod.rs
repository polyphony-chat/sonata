// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use sqlx::{
	PgPool,
	postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::{StdResult, config::DatabaseConfig};

pub(crate) mod api_keys;
pub(crate) mod keytrials;
pub(crate) mod models;
pub(crate) mod serial_number;
pub(crate) mod tokens;

pub(crate) use models::*;

#[derive(Debug, Clone)]
/// Main Database struct. Wrapper around [PgPool].
pub(crate) struct Database {
	/// The underlying `sqlx` [PgPool].
	pub pool: PgPool,
}

impl Database {
	/// Connect to the PostgreSQL Database using configuration options provided
	/// through [DatabaseConfig], which is most commonly derived by parsing a
	/// [SonataConfiguration].
	#[cfg_attr(coverage_nightly, coverage(off))]
	pub async fn connect_with_config(config: &DatabaseConfig) -> StdResult<Self> {
		let connect_options = PgConnectOptions::new()
			.host(&config.host)
			.database(&config.database)
			.application_name("sonata")
			.password(&config.password)
			.port(config.port)
			.ssl_mode(match config.tls {
				crate::config::TlsConfig::Disable => sqlx::postgres::PgSslMode::Disable,
				crate::config::TlsConfig::Allow => sqlx::postgres::PgSslMode::Allow,
				crate::config::TlsConfig::Prefer => sqlx::postgres::PgSslMode::Prefer,
				crate::config::TlsConfig::Require => sqlx::postgres::PgSslMode::Require,
				crate::config::TlsConfig::VerifyCa => sqlx::postgres::PgSslMode::VerifyCa,
				crate::config::TlsConfig::VerifyFull => sqlx::postgres::PgSslMode::VerifyFull,
			})
			.username(&config.username);
		let pool = PgPoolOptions::new()
			.max_connections(config.max_connections)
			.connect_with(connect_options)
			.await?;
		Ok(Self { pool })
	}

	/// Applies the migrations.
	pub(super) async fn run_migrations(&self) -> StdResult<()> {
		sqlx::migrate!().run(&self.pool).await.map_err(|e| e.into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::config::TlsConfig;

	#[test]
	fn test_database_debug() {
		// We can't easily test the actual Database struct without a real connection,
		// but we can test that it implements Debug
		// This is a compile-time test to ensure Debug is implemented
		fn assert_debug<T: std::fmt::Debug>() {}
		assert_debug::<Database>();
	}

	#[test]
	fn test_database_clone() {
		// This is a compile-time test to ensure Clone is implemented
		fn assert_clone<T: Clone>() {}
		assert_clone::<Database>();
	}

	#[tokio::test]
	async fn test_connect_with_config_invalid() {
		let config = DatabaseConfig {
			max_connections: 1,
			database: "nonexistent".to_owned(),
			username: "invalid".to_owned(),
			password: "invalid".to_owned(),
			port: 5432,
			host: "invalid_host".to_owned(),
			tls: TlsConfig::Disable,
		};

		// This should fail to connect
		let result = Database::connect_with_config(&config).await;
		assert!(result.is_err());
	}

	#[tokio::test]
	async fn test_connect_with_config_zero_max_connections() {
		let config = DatabaseConfig {
			max_connections: 0, // Zero connections should cause a panic during pool creation
			database: "test".to_owned(),
			username: "test".to_owned(),
			password: "test".to_owned(),
			port: 5432,
			host: "localhost".to_owned(),
			tls: TlsConfig::Disable,
		};

		// This should panic or error due to zero max_connections
		let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
			tokio::runtime::Runtime::new()
				.unwrap()
				.block_on(async { Database::connect_with_config(&config).await })
		}));
		assert!(result.is_err());
	}
}

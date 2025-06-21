// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::StdResult;
use crate::config::DatabaseConfig;

#[derive(Debug, Clone)]
pub(crate) struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Connect to the PostgreSQL Database using configuration options provided through [DatabaseConfig],
    /// which is most commonly derived by parsing a [SonataConfiguration].
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
}

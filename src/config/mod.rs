// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::Deref;
use std::sync::OnceLock;

use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

use crate::{StdError, StdResult};

static CONFIG: OnceLock<SonataConfig> = OnceLock::new();

const TLS_CONFIG_DISABLE: &str = "disable";
const TLS_CONFIG_ALLOW: &str = "allow";
const TLS_CONFIG_PREFER: &str = "prefer";
const TLS_CONFIG_REQUIRE: &str = "require";
const TLS_CONFIG_VERIFY_CA: &str = "verify_ca";
const TLS_CONFIG_VERIFY_FULL: &str = "verify_full";

#[derive(Deserialize, Debug)]
pub struct SonataConfig {
    pub api: ApiConfig,
    pub gateway: GatewayConfig,
    pub general: GeneralConfig,
}

#[derive(Deserialize, Debug)]
pub struct ApiConfig {
    #[serde(flatten)]
    config: ComponentConfig,
}

impl Deref for ApiConfig {
    type Target = ComponentConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

#[derive(Deserialize, Debug)]
pub struct GatewayConfig {
    #[serde(flatten)]
    config: ComponentConfig,
}

impl Deref for GatewayConfig {
    type Target = ComponentConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    pub log_level: String,
    pub database: DatabaseConfig,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub database: String,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    pub tls: TlsConfig,
}

#[derive(Deserialize, Debug)]
pub struct ComponentConfig {
    pub enabled: bool,
    pub port: u16,
    pub host: String,
    pub tls: bool,
}

impl SonataConfig {
    pub fn init(input: &str) -> StdResult<()> {
        let cfg = toml::from_str::<Self>(input)?;
        CONFIG.set(cfg).map_err(|_| String::from("config global was already set"))?;
        Ok(())
    }

    #[allow(clippy::expect_used)]
    /// Gets a static reference to the parsed configuration file. Will panic, if [Self] has not been initialized using [Self::init()].
    pub fn get_or_panic() -> &'static Self {
        CONFIG.get().expect("config has not been initialized yet")
    }
}

#[derive(Debug, Deserialize, Default)]
/// TLS configuration modes. Also called `sslconfig` by PostgreSQL. See <https://www.postgresql.org/docs/current/libpq-ssl.html#:~:text=32.1.%C2%A0SSL%20Mode-,descriptions,-sslmode>
/// for the security implications of this choice.
pub enum TlsConfig {
    /// I don't care about security, and I don't want to pay the overhead of
    /// encryption.
    Disable,
    /// I don't care about security, but I will pay the overhead of encryption
    /// if the server insists on it.
    Allow,
    /// I don't care about encryption, but I wish to pay the overhead of
    /// encryption if the server supports it.
    Prefer,
    /// I want my data to be encrypted, and I accept the overhead. I trust that
    /// the network will make sure I always connect to the server I want.
    #[default]
    Require,
    /// I want my data encrypted, and I accept the overhead. I want to be sure
    /// that I connect to a server that I trust.
    VerifyCa,
    /// I want my data encrypted, and I accept the overhead. I want to be sure
    /// that I connect to a server I trust, and that it's the one I specify.
    VerifyFull,
}

impl TryFrom<&str> for TlsConfig {
    type Error = StdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            TLS_CONFIG_DISABLE => Ok(Self::Disable),
            TLS_CONFIG_ALLOW => Ok(Self::Disable),
            TLS_CONFIG_PREFER => Ok(Self::Disable),
            TLS_CONFIG_REQUIRE => Ok(Self::Disable),
            "verifyca" | TLS_CONFIG_VERIFY_CA | "verify-ca" => Ok(Self::Disable),
            "verifyfull" | TLS_CONFIG_VERIFY_FULL | "verify-full" => Ok(Self::Disable),
            other => Err(format!("{other} is not a valid TLS configuration value").into()),
        }
    }
}

impl std::fmt::Display for TlsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TlsConfig::Disable => TLS_CONFIG_DISABLE,
            TlsConfig::Allow => TLS_CONFIG_ALLOW,
            TlsConfig::Prefer => TLS_CONFIG_PREFER,
            TlsConfig::Require => TLS_CONFIG_REQUIRE,
            TlsConfig::VerifyCa => TLS_CONFIG_VERIFY_CA,
            TlsConfig::VerifyFull => TLS_CONFIG_VERIFY_FULL,
        })
    }
}

impl std::str::FromStr for TlsConfig {
    type Err = Box<dyn std::error::Error + 'static>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TlsConfig::try_from(s)
    }
}

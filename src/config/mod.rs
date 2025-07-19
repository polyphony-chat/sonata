// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{ops::Deref, sync::OnceLock};

use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

use crate::{StdError, StdResult};

/// Module-private "global" variable for storing the configuration values once
/// they are parsed.
static CONFIG: OnceLock<SonataConfig> = OnceLock::new();

/// PostgreSQL: TLS Disabled
const TLS_CONFIG_DISABLE: &str = "disable";
/// PostgreSQL: TLS Allowed
const TLS_CONFIG_ALLOW: &str = "allow";
/// PostgreSQL: TLS Preferred
const TLS_CONFIG_PREFER: &str = "prefer";
/// PostgreSQL: TLS Required
const TLS_CONFIG_REQUIRE: &str = "require";
/// PostgreSQL: TLS Required with TLS certificate authority verification
const TLS_CONFIG_VERIFY_CA: &str = "verify_ca";
/// PostgreSQL: TLS Required with TLS certificate authority and subject
/// verification
const TLS_CONFIG_VERIFY_FULL: &str = "verify_full";

#[derive(Deserialize, Debug, Clone)]
/// The `sonata.toml` configuration file as Rust structs.
pub struct SonataConfig {
    /// API module configuration
    pub api: ApiConfig,
    /// Gateway module configuration
    pub gateway: GatewayConfig,
    /// General configuration, mostly consisting of [DatabaseConfig]
    pub general: GeneralConfig,
}

#[derive(Deserialize, Debug, Clone)]
/// API Module configuration
pub struct ApiConfig {
    #[serde(flatten)]
    /// [ComponentConfig], holding the configuration values
    config: ComponentConfig,
}

impl Deref for ApiConfig {
    type Target = ComponentConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

#[derive(Deserialize, Debug, Clone)]
/// Gateway module configuration
pub struct GatewayConfig {
    #[serde(flatten)]
    /// [ComponentConfig], holding the configuration values
    config: ComponentConfig,
}

impl Deref for GatewayConfig {
    type Target = ComponentConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

#[derive(Deserialize, Debug, Clone)]
/// General configuration, consisting of database configuration
pub struct GeneralConfig {
    /// Database configuration, including host, port, password, etc.
    pub database: DatabaseConfig,
    /// The domain of this Sonata server instance.
    pub server_domain: String,
}

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    /// How many connections to allocate for this connection pool at maximum.
    /// PostgreSQLs default value is 100.
    pub max_connections: u32,
    /// The name of the database to connect to.
    pub database: String,
    /// The username with which to connect to the database to.
    pub username: String,
    /// The password with which to connect to the database to.
    pub password: String,
    /// The port on which the database is listening on.
    pub port: u16,
    /// The host URL/IP which the database is listening on.
    pub host: String,
    #[serde(default)]
    #[serde_as(as = "DisplayFromStr")]
    /// TLS connection settings for the database.
    pub tls: TlsConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComponentConfig {
    /// Whether this component is enabled.
    pub enabled: bool,
    /// Which port to bind to.
    pub port: u16,
    /// Which host address to bind to.
    pub host: String,
    /// Whether TLS is enabled or not.
    pub tls: bool,
}

impl SonataConfig {
    /// Initializes the [SonataConfig] by reading the configuration file, then
    /// storing it in a global variable. After calling this function
    /// successfully, the configuration may be retrieved at any time by calling
    /// `SonataConfig::get_or_panic()`.
    ///
    /// This function may only be called once. Subsequent calls of this function
    /// will yield an Error.
    pub fn init(input: &str) -> StdResult<()> {
        let cfg = toml::from_str::<Self>(input)?;
        CONFIG.set(cfg).map_err(|_| String::from("config global was already set"))?;
        Ok(())
    }

    #[allow(clippy::expect_used)]
    /// Gets a static reference to the parsed configuration file. Will panic, if
    /// [Self] has not been initialized using [Self::init()].
    pub fn get_or_panic() -> &'static Self {
        CONFIG.get().expect("config has not been initialized yet")
    }
}

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
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
            TLS_CONFIG_ALLOW => Ok(Self::Allow),
            TLS_CONFIG_PREFER => Ok(Self::Prefer),
            TLS_CONFIG_REQUIRE => Ok(Self::Require),
            "verifyca" | TLS_CONFIG_VERIFY_CA | "verify-ca" => Ok(Self::VerifyCa),
            "verifyfull" | TLS_CONFIG_VERIFY_FULL | "verify-full" => Ok(Self::VerifyFull),
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
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TlsConfig::try_from(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_try_from() {
        // Test valid configurations
        assert!(matches!(TlsConfig::try_from("disable"), Ok(TlsConfig::Disable)));
        assert!(matches!(TlsConfig::try_from("allow"), Ok(TlsConfig::Allow)));
        assert!(matches!(TlsConfig::try_from("prefer"), Ok(TlsConfig::Prefer)));
        assert!(matches!(TlsConfig::try_from("require"), Ok(TlsConfig::Require)));
        assert!(matches!(TlsConfig::try_from("verify_ca"), Ok(TlsConfig::VerifyCa)));
        assert!(matches!(TlsConfig::try_from("verify-ca"), Ok(TlsConfig::VerifyCa)));
        assert!(matches!(TlsConfig::try_from("verifyca"), Ok(TlsConfig::VerifyCa)));
        assert!(matches!(TlsConfig::try_from("verify_full"), Ok(TlsConfig::VerifyFull)));
        assert!(matches!(TlsConfig::try_from("verify-full"), Ok(TlsConfig::VerifyFull)));
        assert!(matches!(TlsConfig::try_from("verifyfull"), Ok(TlsConfig::VerifyFull)));

        // Test case insensitivity
        assert!(matches!(TlsConfig::try_from("DISABLE"), Ok(TlsConfig::Disable)));
        assert!(matches!(TlsConfig::try_from("DiSaBlE"), Ok(TlsConfig::Disable)));

        // Test invalid configuration
        assert!(TlsConfig::try_from("invalid").is_err());
        assert!(TlsConfig::try_from("").is_err());
        assert!(TlsConfig::try_from("random_value").is_err());
    }

    #[test]
    fn test_tls_config_display() {
        assert_eq!(TlsConfig::Disable.to_string(), "disable");
        assert_eq!(TlsConfig::Allow.to_string(), "allow");
        assert_eq!(TlsConfig::Prefer.to_string(), "prefer");
        assert_eq!(TlsConfig::Require.to_string(), "require");
        assert_eq!(TlsConfig::VerifyCa.to_string(), "verify_ca");
        assert_eq!(TlsConfig::VerifyFull.to_string(), "verify_full");
    }

    #[test]
    fn test_tls_config_from_str() {
        // Test that FromStr trait works correctly (delegates to TryFrom)
        assert!(matches!("disable".parse::<TlsConfig>(), Ok(TlsConfig::Disable)));
        assert!(matches!("allow".parse::<TlsConfig>(), Ok(TlsConfig::Allow)));
        assert!(matches!("prefer".parse::<TlsConfig>(), Ok(TlsConfig::Prefer)));
        assert!(matches!("require".parse::<TlsConfig>(), Ok(TlsConfig::Require)));
        assert!(matches!("verify_ca".parse::<TlsConfig>(), Ok(TlsConfig::VerifyCa)));
        assert!(matches!("verify_full".parse::<TlsConfig>(), Ok(TlsConfig::VerifyFull)));
        assert!("invalid".parse::<TlsConfig>().is_err());
    }

    #[test]
    fn test_tls_config_default() {
        // Test that the default is Require
        assert!(matches!(TlsConfig::default(), TlsConfig::Require));
    }

    #[test]
    fn test_api_config_deref() {
        let config = ApiConfig {
            config: ComponentConfig {
                enabled: true,
                port: 8080,
                host: "localhost".to_owned(),
                tls: true,
            },
        };

        // Test that deref works correctly
        assert!(config.enabled);
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "localhost");
        assert!(config.tls);
    }

    #[test]
    fn test_gateway_config_deref() {
        let config = GatewayConfig {
            config: ComponentConfig {
                enabled: false,
                port: 9090,
                host: "0.0.0.0".to_owned(),
                tls: false,
            },
        };

        // Test that deref works correctly
        assert!(!config.enabled);
        assert_eq!(config.port, 9090);
        assert_eq!(config.host, "0.0.0.0");
        assert!(!config.tls);
    }

    #[test]
    fn test_sonata_config_init() {
        let toml_str =
            &std::fs::read_to_string(format!("{}/sonata.toml", std::env!("CARGO_MANIFEST_DIR")))
                .unwrap();

        let _config: SonataConfig = toml::from_str(&toml_str).unwrap();

        // First init should succeed
        assert!(SonataConfig::init(toml_str).is_ok());

        // Second init should fail (already initialized)
        assert!(SonataConfig::init(toml_str).is_err());
    }

    #[test]
    fn test_sonata_config_init_invalid_toml() {
        let invalid_toml = "this is not valid toml";
        assert!(SonataConfig::init(invalid_toml).is_err());
    }

    #[test]
    fn test_sonata_config_init_missing_fields() {
        let incomplete_toml = r#"
[api]
enabled = true
# missing required fields
"#;
        assert!(SonataConfig::init(incomplete_toml).is_err());
    }

    #[test]
    #[should_panic(expected = "config has not been initialized yet")]
    fn test_sonata_config_get_or_panic_without_init() {
        // Clear the global state if it exists
        SonataConfig::get_or_panic();
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::Deref;

use serde::Deserialize;

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

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig;

#[derive(Deserialize, Debug)]
pub struct ComponentConfig {
    pub enabled: bool,
    pub port: u16,
    pub host: String,
    pub tls: bool,
}

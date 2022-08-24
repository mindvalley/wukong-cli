use env_config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Api {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Okta {
    pub client_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub api: Api,
    pub okta: Okta,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let build_mode = env::var("WUKONG_BUILD_MODE").unwrap_or_else(|_| "dev".into());

        let s = Config::builder()
            .add_source(File::with_name(&format!("src/env/{}.toml", build_mode)))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("src/env/local.toml").required(false))
            // Add in settings from the environment (with a prefix of WUKONG)
            // Eg.. `WUKONG_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("WUKONG").separator("_"))
            .build()?;

        s.try_deserialize()
    }
}

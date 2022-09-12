use crate::{
    clap_app::ClapApp,
    config::{Config, CONFIG_FILE},
    error::{CliError, ConfigError},
};
use clap::Parser;

pub enum ConfigState {
    InitialisedButUnAuthenticated(Config),
    InitialisedAndAuthenticated(Config),
    Uninitialised,
}

pub struct App {
    pub config: ConfigState,
    pub cli: ClapApp,
}

impl App {
    pub fn new<'a>() -> Result<Self, CliError<'a>> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = match Config::load(config_file) {
            Ok(config) => {
                if config.auth.is_none() {
                    ConfigState::InitialisedButUnAuthenticated(config)
                } else {
                    ConfigState::InitialisedAndAuthenticated(config)
                }
            }
            Err(error) => match error {
                CliError::ConfigError(ConfigError::NotFound { .. }) => ConfigState::Uninitialised,
                _ => return Err(error),
            },
        };

        Ok(Self {
            config,
            cli: ClapApp::parse(),
        })
    }
}

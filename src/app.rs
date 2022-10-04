use crate::{
    clap_app::ClapApp,
    config::Config,
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
    pub fn new<'a>(config_file: &'a str) -> Result<Self, CliError<'a>> {
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

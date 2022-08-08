use crate::{
    clap_app::ClapApp,
    config::{Config, CONFIG_FILE},
    error::CliError,
};
use clap::Parser;

pub enum ConfigState {
    Initialized(Config),
    Uninitialized,
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
                    return Err(CliError::UnAuthenticated);
                }

                ConfigState::Initialized(config)
            }
            Err(error) => match error {
                CliError::ConfigError(ref config_error) => match config_error {
                    crate::error::ConfigError::NotFound { .. } => ConfigState::Uninitialized,
                    _ => return Err(error),
                },
                _ => return Err(error),
            },
        };

        Ok(Self {
            config,
            cli: ClapApp::parse(),
        })
    }
}

use crate::{
    clap_app::ClapApp,
    config::Config,
    error::{CliError, ConfigError},
};
use clap::Parser;
use once_cell::sync::OnceCell;

pub enum ConfigState {
    InitialisedButUnAuthenticated(Config),
    InitialisedAndAuthenticated(Config),
    Uninitialised,
}

pub struct App {
    pub config: ConfigState,
    pub cli: ClapApp,
}

#[derive(Debug)]
pub struct AppState {
    pub api_url: String,
    pub okta_client_id: String,
}

pub static APP_STATE: OnceCell<AppState> = OnceCell::new();

impl App {
    pub fn new(config_file: &'static str) -> Result<Self, CliError> {
        let config = match Config::load(config_file) {
            Ok(config) => {
                APP_STATE
                    .set(AppState {
                        api_url: config.core.wukong_api_url.clone(),
                        okta_client_id: config.core.okta_client_id.clone(),
                    })
                    .unwrap();

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

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
                // get values from config file if the config is initialised
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
            Err(error) => {
                // use default values if the config is not initialised
                APP_STATE
                    .set(AppState {
                        #[cfg(all(feature = "prod"))]
                        api_url: "https://wukong-api-proxy.mindvalley.dev/api".to_string(),
                        #[cfg(not(feature = "prod"))]
                        api_url: "http://localhost:4000/api".to_string(),

                        okta_client_id: "0oakfxaegyAV5JDD5357".to_string(),
                    })
                    .unwrap();

                match error {
                    CliError::ConfigError(ConfigError::NotFound { .. }) => {
                        ConfigState::Uninitialised
                    }
                    _ => return Err(error),
                }
            }
        };

        Ok(Self {
            config,
            cli: ClapApp::parse(),
        })
    }
}

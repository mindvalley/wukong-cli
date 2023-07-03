use crate::{config::Config as CliConfig, error::WKCliError};

use super::ConfigName;

pub fn handle_get(config_name: &ConfigName) -> Result<bool, WKCliError> {
    let config = CliConfig::load()?;
    match config_name {
        ConfigName::Application => println!("{}", config.core.application),
        ConfigName::WukongApiUrl => println!("{}", config.core.wukong_api_url),
        ConfigName::OktaClientId => println!("{}", config.core.okta_client_id),
    };
    Ok(true)
}

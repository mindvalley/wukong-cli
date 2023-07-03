use crate::{config::Config as CliConfig, error::WKCliError};

use super::ConfigName;

pub fn handle_set(config_name: &ConfigName, config_value: &str) -> Result<bool, WKCliError> {
    let mut config = CliConfig::load()?;
    match config_name {
        ConfigName::Application => {
            config.core.application = config_value.trim().to_string();
            config.save_to_exisiting_file()?;
            println!("Updated property [core/application].");
        }
        ConfigName::WukongApiUrl => {
            config.core.wukong_api_url = config_value.trim().to_string();
            config.save_to_exisiting_file()?;
            println!("Updated property [core/wukong_api_url].");
        }
        ConfigName::OktaClientId => {
            config.core.okta_client_id = config_value.trim().to_string();
            config.save_to_exisiting_file()?;
            println!("Updated property [core/okta_client_id].");
        }
    };
    Ok(true)
}

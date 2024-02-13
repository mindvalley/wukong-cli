use crate::{config::Config as CliConfig, error::WKCliError};

use super::ConfigName;

pub fn handle_get(config_name: &ConfigName) -> Result<bool, WKCliError> {
    let config = CliConfig::load_from_default_path()?;
    println!("{:?}", config_name);
    match config_name {
        ConfigName::Application => println!("{}", config.core.application),
        ConfigName::WukongApiUrl => println!("{}", config.core.wukong_api_url),
        ConfigName::OktaClientId => println!(
            "{}",
            if let Some(okta) = config.auth.okta {
                okta.client_id
            } else {
                "Okta client id not set. Please use `wukong login`".to_string()
            }
        ),
    };
    Ok(true)
}

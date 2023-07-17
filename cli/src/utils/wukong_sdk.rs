use wukong_sdk::{WKClient, WKConfig};

use crate::config::Config;

pub trait FromWKCliConfig {
    fn from_cli_config(config: &Config) -> Self;
}

impl FromWKCliConfig for WKClient {
    fn from_cli_config(config: &Config) -> Self {
        WKClient::new(WKConfig {
            api_url: config.core.wukong_api_url.clone(),
            access_token: config.auth.clone().map(|auth| auth.id_token),
        })
    }
}

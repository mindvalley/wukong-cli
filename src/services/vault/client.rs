use log::debug;

use crate::{error::CliError, Config as CLIConfig};

pub struct VaultClient {
    api_key: String,
}

impl VaultClient {
    // Make api_key opttional:
    pub fn new(api_key: Option<&str>) -> Self {
        Self {
            api_key: api_key.unwrap_or("").to_string(),
        }
    }

    pub async fn login(&self, mut _current_config: CLIConfig) -> Result<bool, CliError> {
        debug!("Authenticating with the vault server ...");

        // TODO: Implement the login logic here

        Ok(true)
    }

    async fn validate_user(&self) -> Result<bool, CliError> {
        Ok(true)
    }

    pub async fn fetch_lists(&self) -> Result<bool, CliError> {
        self.validate_user().await?;

        let _client = reqwest::Client::new();

        // TODO: Make client call:

        print!("Hello world");

        Ok(true)
    }
}

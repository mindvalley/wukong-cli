use crate::config::VaultConfig;
use crate::error::APIError;
use crate::output::colored_println;
use crate::{config::CONFIG_FILE, error::CliError, Config as CLIConfig};
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;

use super::client::VaultClient;

pub struct Vault {
    api_key: Option<String>,
}

impl Default for VaultClient {
    fn default() -> Self {
        Self::new()
    }
}

struct ConfigWithPath {
    config: CLIConfig,
    path: String,
}

impl Vault {
    pub fn new(api_key: Option<&str>) -> Self {
        let api_key_string = api_key.map(|s| s.to_string());

        Self {
            api_key: api_key_string,
        }
    }

    fn get_config_with_path(&self) -> Result<ConfigWithPath, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = CLIConfig::load(config_file)?;

        let config_with_path = ConfigWithPath {
            config,
            path: config_file.to_string(),
        };

        Ok(config_with_path)
    }

    pub async fn handle_login(&self) -> Result<bool, CliError> {
        debug!("Authenticating with the vault server ...");

        let mut email: Option<String> = None;
        let mut config_with_path = self.get_config_with_path().unwrap();

        if let Some(vault_config) = &config_with_path.config.vault {
            let selections = vec!["Use the existing account", "Log in with a new account"];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    format!(
                "You have previously logged in as \"{}\". Would you like to continue using this account?",
                vault_config.email
            ))
                .default(0)
                .items(&selections[..])
                .interact()?;

            match selection {
                0 => {
                    colored_println!("Continuing with \"{}\"...", vault_config.email);

                    email = Some(vault_config.email.to_string())
                }
                1 => {
                    let input_email: String =
                        dialoguer::Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Please enter your email address")
                            .interact()?;

                    email = Some(input_email.trim().to_string());
                }
                _ => panic!("Invalid selection"),
            }
        }

        // If the user has not logged in before, or has chosen to log in with a new account:
        if email.is_none() {
            let input_email: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Please enter your email address")
                .interact()?;

            email = Some(input_email.trim().to_string());
        }

        let password = dialoguer::Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your password")
            .interact()?;

        let vault_client = VaultClient::new();

        // Make login request:
        match vault_client
            .login(&email.clone().unwrap(), password.as_str())
            .await
        {
            Ok(data) => {
                config_with_path.config.vault = Some(VaultConfig {
                    api_key: data.auth.client_token,
                    email: email.clone().unwrap(),
                });

                config_with_path
                    .config
                    .save(&config_with_path.path)
                    .unwrap();

                colored_println!("You are now logged in as {}.", "mohamed".to_string());
            }
            Err(e) => {
                debug!("Error: {:?}", e);

                if e.status().unwrap() == 400 {
                    return Err(CliError::AuthenticationFailed);
                } else {
                    colored_println!("An error occurred. Please try again.");
                    return Err(CliError::APIError(APIError::ResponseError {
                        code: e.status().unwrap().to_string(),
                        message: e.to_string(),
                    }));
                }
            }
        };

        Ok(true)
    }
}

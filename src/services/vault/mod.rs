pub mod client;

use crate::config::VaultConfig;
use crate::error::APIError;
use crate::output::colored_println;
use crate::{config::CONFIG_FILE, error::CliError, Config as CLIConfig};
use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;

use self::client::VaultClient;

impl Default for VaultClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

struct ConfigWithPath {
    config: CLIConfig,
    path: String,
}

pub struct Vault {}

impl Vault {
    pub fn new() -> Self {
        Self {}
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

                colored_println!("You are now logged in as {}.", email.unwrap());
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

    pub async fn get_lists(&self) -> Result<bool, CliError> {
        let api_key = &self.get_token(false).await?;

        let vault_client = VaultClient::new();

        match vault_client
            .fetch_lists(api_key, "engineering/fastly/staging")
            .await
        {
            Ok(data) => {
                println!("{:?}", data);
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
        }

        Ok(true)
    }

    #[async_recursion]
    async fn get_token(&self, skip_verify: bool) -> Result<String, CliError> {
        debug!("Getting token ...");

        let config_with_path = self.get_config_with_path().unwrap();
        let mut api_key = match &config_with_path.config.vault {
            Some(vault_config) => vault_config.api_key.clone(),
            None => {
                debug!("No token found in config file. Prompting user to log in ...");
                colored_println!("You are not logged in. Please log in to continue.");

                self.handle_login().await?;
                self.get_token(true).await? // Set the skip_verify flag to true on recursive call
            }
        };

        if !skip_verify {
            self.verify_token(&api_key).await?;
            api_key = self.get_token(true).await?;
        }

        Ok(api_key)
    }

    async fn verify_token(&self, api_key: &str) -> Result<bool, CliError> {
        debug!("Verifying token ...");
        let vault_client = VaultClient::new();

        match vault_client.verify_token(api_key).await {
            Ok(data) => {
                debug!("Token verified: {:?}", data);
            }
            Err(e) => {
                if e.status().unwrap() == 403 {
                    // User is asked to re-login if the token is invalid
                    colored_println!("Your login session has expired. Please log in again.");
                    self.handle_login().await?;
                } else {
                    debug!("Error: {:?}", e);
                    colored_println!("An error occurred. Please try again.");
                    return Err(CliError::APIError(APIError::ResponseError {
                        code: e.status().unwrap().to_string(),
                        message: e.to_string(),
                    }));
                }
            }
        }

        Ok(true)
    }
}

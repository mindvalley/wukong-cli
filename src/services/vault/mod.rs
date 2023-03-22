pub mod client;

use crate::config::VaultConfig;
use crate::error::VaultError;
use crate::loader::new_spinner_progress_bar;
use crate::output::colored_println;
use crate::services::vault::client::{FetchSecrets, UpdateSecret, VerifyToken};
use crate::Config as CLIConfig;
use async_recursion::async_recursion;
use dialoguer::theme::ColorfulTheme;
use log::debug;
use reqwest::StatusCode;

use self::client::{FetchSecretsData, Login, VaultClient};

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

pub struct Vault {
    vault_client: VaultClient,
}

impl Vault {
    pub fn new() -> Self {
        Self {
            vault_client: VaultClient::new(),
        }
    }

    pub async fn handle_login(&self) -> Result<bool, VaultError> {
        let mut email: Option<String> = None;
        let mut config_with_path = CLIConfig::get_config_with_path().unwrap();

        if let Some(vault_config) = &config_with_path.config.auth {
            colored_println!("Continuing with \"{}\"...", vault_config.account);
            email = Some(vault_config.account.to_string())
        }

        if email.is_none() {
            debug!("No email found in config file");
            return Err(VaultError::UnInitialised);
        }

        let password = dialoguer::Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your password")
            .interact()?;

        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Authenticating the user...");

        let response = self
            .vault_client
            .login(&email.clone().unwrap(), password.as_str())
            .await?;

        progress_bar.finish_and_clear();

        if response.status().is_success() {
            let data = response.json::<Login>().await?;

            config_with_path.config.vault = Some(VaultConfig {
                api_key: data.auth.client_token,
            });

            config_with_path
                .config
                .save(&config_with_path.path)
                .unwrap();

            colored_println!("You are now logged in as {}.", email.unwrap());
        } else {
            self.handle_error(response).await?;
        }

        Ok(true)
    }

    pub async fn get_secret(&self, path: &str, key: &str) -> Result<String, VaultError> {
        let secrets = self.get_secrets(path).await?;

        // Extract the secret from the response:
        let secret = secrets.data.get(key).ok_or(VaultError::SecretNotFound)?;

        Ok(secret.to_string())
    }

    pub async fn get_secrets(&self, path: &str) -> Result<FetchSecretsData, VaultError> {
        let api_key = &self.get_token(false).await?;

        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Fetching secrets... ");

        let response = self.vault_client.fetch_secrets(api_key, path).await?;

        progress_bar.finish_and_clear();

        if response.status().is_success() {
            let secrets = response.json::<FetchSecrets>().await?;
            Ok(secrets.data)
        } else {
            self.handle_error(response).await?;
            unreachable!()
        }
    }

    pub async fn update_secret(
        &self,
        path: &str,
        key: &str,
        value: &str,
    ) -> Result<bool, VaultError> {
        let api_key = &self.get_token(false).await?;

        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Updating secrets... ");

        let response = self
            .vault_client
            .update_secret(api_key, path, key, value)
            .await?;

        progress_bar.finish_and_clear();

        if response.status().is_success() {
            colored_println!("Successfully updated the secrets.");
        } else {
            self.handle_error(response).await?;
        }

        Ok(true)
    }

    #[async_recursion]
    async fn get_token(&self, skip_verify: bool) -> Result<String, VaultError> {
        debug!("Getting token ...");

        let config_with_path = CLIConfig::get_config_with_path().unwrap();
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

    async fn verify_token(&self, api_key: &str) -> Result<bool, VaultError> {
        debug!("Verifying token ...");

        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Verifying the token...");

        let response = self.vault_client.verify_token(api_key).await?;
        progress_bar.finish_and_clear();

        if !response.status().is_success() {
            self.handle_error(response).await?;
        }

        Ok(true)
    }

    #[async_recursion]
    async fn handle_error(&self, response: reqwest::Response) -> Result<(), VaultError> {
        debug!("Error: {:?}", response);

        let status = response.status();
        let message = response.text().await?;

        match status {
            StatusCode::NOT_FOUND => {
                debug!("The requested resource was not found.");
                return Err(VaultError::SecretNotFound);
            }
            StatusCode::FORBIDDEN => {
                colored_println!("Your login session has expired/invalid. Please log in again.");
                self.handle_login().await?;
            }
            StatusCode::BAD_REQUEST => {
                if message.contains("Okta auth failed") {
                    colored_println!("Invalid credentials. Please try again.");
                    return Err(VaultError::AuthenticationFailed);
                } else {
                    colored_println!("Bad request. Please try again.");
                    return Err(VaultError::ResponseError {
                        code: status.to_string(),
                        message,
                    });
                }
            }
            _ => {
                colored_println!("Error: {}", message);
                return Err(VaultError::ResponseError {
                    code: status.to_string(),
                    message,
                });
            }
        };

        Ok(())
    }
}

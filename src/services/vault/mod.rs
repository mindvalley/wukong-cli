pub mod client;

use std::collections::HashMap;

use crate::config::VaultConfig;
use crate::error::VaultError;
use crate::loader::new_spinner_progress_bar;
use crate::output::colored_println;
use crate::services::vault::client::FetchSecrets;
use crate::Config as CLIConfig;
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

    pub async fn get_token_or_login(&self) -> Result<String, VaultError> {
        let mut token = match self.get_token().await {
            Ok(token) => token,
            Err(VaultError::ApiTokenNotFound) => self.handle_login().await?,
            Err(err) => return Err(err),
        };

        match self.is_valid_token(&token).await {
            Ok(_) => Ok(token),
            Err(VaultError::ApiTokenInvalid) => {
                token = self.handle_login().await?;
                Ok(token)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn handle_login(&self) -> Result<String, VaultError> {
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
                api_token: data.auth.client_token.clone(),
            });

            config_with_path
                .config
                .save(&config_with_path.path)
                .unwrap();

            colored_println!("You are now logged in as {}.", email.unwrap());

            Ok(data.auth.client_token)
        } else {
            self.handle_error(response).await?;
            unreachable!()
        }
    }

    pub async fn get_secret(
        &self,
        api_token: &str,
        path: &str,
        key: &str,
    ) -> Result<String, VaultError> {
        let secrets = self.get_secrets(api_token, path).await?;
        let secret = secrets.data.get(key).ok_or(VaultError::SecretNotFound)?;

        Ok(secret.to_string())
    }

    pub async fn get_secrets(
        &self,
        api_token: &str,
        path: &str,
    ) -> Result<FetchSecretsData, VaultError> {
        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Fetching secrets... ");

        let response = self.vault_client.fetch_secrets(api_token, path).await?;

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
        api_token: &str,
        path: &str,
        data: &HashMap<&str, &str>,
    ) -> Result<bool, VaultError> {
        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Updating secrets... ");

        let response = self
            .vault_client
            .update_secret(api_token, path, data)
            .await?;

        progress_bar.finish_and_clear();

        if response.status().is_success() {
            colored_println!("Successfully updated the secrets.");
        } else {
            self.handle_error(response).await?;
        }

        Ok(true)
    }

    async fn get_token(&self) -> Result<String, VaultError> {
        debug!("Getting token ...");

        let config_with_path = CLIConfig::get_config_with_path().unwrap();
        let api_token = match &config_with_path.config.vault {
            Some(vault_config) => vault_config.api_token.clone(),
            None => {
                return Err(VaultError::ApiTokenNotFound);
            }
        };

        Ok(api_token)
    }

    async fn is_valid_token(&self, api_token: &str) -> Result<bool, VaultError> {
        debug!("Verifying token ...");

        let progress_bar = new_spinner_progress_bar();
        progress_bar.set_message("Verifying the token...");

        let response = self.vault_client.verify_token(api_token).await?;
        progress_bar.finish_and_clear();

        if !response.status().is_success() {
            self.handle_error(response).await?;
        }

        Ok(true)
    }

    async fn handle_error(&self, response: reqwest::Response) -> Result<(), VaultError> {
        debug!("Error: {:?}", response);

        let status = response.status();
        let message = response.text().await?;

        match status {
            StatusCode::NOT_FOUND => Err(VaultError::SecretNotFound),
            StatusCode::FORBIDDEN => Err(VaultError::ApiTokenInvalid),
            StatusCode::BAD_REQUEST => {
                if message.contains("Okta auth failed") {
                    Err(VaultError::AuthenticationFailed)
                } else {
                    Err(VaultError::ResponseError {
                        code: status.to_string(),
                        message,
                    })
                }
            }
            _ => Err(VaultError::ResponseError {
                code: status.to_string(),
                message,
            }),
        }
    }
}

pub mod client;

use self::client::{FetchSecretsData, VaultClient};
use crate::error::{APIError, VaultError, WKError};
use crate::services::vault::client::FetchSecrets;
use crate::WKClient;
use log::debug;
use reqwest::StatusCode;
use std::collections::HashMap;

async fn handle_error(response: reqwest::Response) -> Result<(), VaultError> {
    debug!("Error: {:?}", response);

    let status = response.status();
    let message = response.text().await?;

    match status {
        StatusCode::NOT_FOUND => Err(VaultError::SecretNotFound),
        StatusCode::FORBIDDEN => Err(VaultError::PermissionDenied),
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

/// Functions from Vault service.
impl WKClient {
    /// Get secrets from Vault.
    ///
    /// It will return [`WKError::VaultError`] if the response is not success.
    pub async fn get_secrets(
        &self,
        api_token: &str,
        path: &str,
    ) -> Result<FetchSecretsData, WKError> {
        let vault_client = VaultClient::new();
        let response = vault_client
            .fetch_secrets(api_token, path)
            .await
            .map_err(<reqwest::Error as Into<APIError>>::into)?;

        if response.status().is_success() {
            let secrets = response
                .json::<FetchSecrets>()
                .await
                .map_err(<reqwest::Error as Into<APIError>>::into)?;

            Ok(secrets.data)
        } else {
            handle_error(response).await?;
            unreachable!()
        }
    }

    /// Get secret by `key` value from Vault.
    ///
    /// It will return [`WKError::VaultError`] if the response is not success.
    pub async fn get_secret(
        &self,
        api_token: &str,
        path: &str,
        key: &str,
    ) -> Result<String, WKError> {
        let secrets = self.get_secrets(api_token, path).await?;
        let secret = secrets.data.get(key).ok_or(VaultError::SecretNotFound)?;

        Ok(secret.to_string())
    }

    /// Update secret on Vault.
    ///
    /// It will return [`WKError::VaultError`] if the response is not success.
    pub async fn update_secret(
        &self,
        api_token: &str,
        path: &str,
        data: &HashMap<&str, &str>,
    ) -> Result<bool, WKError> {
        let vault_client = VaultClient::new();

        let response = vault_client
            .update_secret(api_token, path, data)
            .await
            .map_err(<reqwest::Error as Into<APIError>>::into)?;

        if !response.status().is_success() {
            handle_error(response).await?;
        }

        Ok(true)
    }
}

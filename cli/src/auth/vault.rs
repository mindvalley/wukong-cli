use aion::*;
use chrono::{DateTime, Duration, Local};
use dialoguer::theme::ColorfulTheme;
use log::debug;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, VaultConfig},
    error::{AuthError, WKCliError},
    loader::new_spinner,
    output::colored_println,
    utils::compare_with_current_time,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub client_token: String,
    pub lease_duration: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Renew {
    pub auth: Auth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub auth: Auth,
}

const LOGIN_URL: &str = "/v1/auth/okta/login";
const RENEW_TOKEN_URL: &str = "/v1/auth/token/renew-self";
const VERIFY_TOKEN_URL: &str = "/v1/auth/token/lookup-self";

pub static BASE_URL: Lazy<String> = Lazy::new(|| {
    #[cfg(feature = "prod")]
    return "https://bunker.mindvalley.dev:8200".to_string();

    #[cfg(not(feature = "prod"))]
    {
        match std::env::var("WUKONG_DEV_VAULT_API_URL") {
            Ok(vault_api_url) => vault_api_url,
            Err(_) => "https://bunker.mindvalley.dev:8200".to_string(),
        }
    }
});

pub async fn get_token_or_login(config: &mut Config) -> Result<String, WKCliError> {
    let auth_config = config.auth.as_ref().ok_or(WKCliError::UnAuthenticated)?;
    let client = reqwest::Client::new();

    match &config.vault {
        Some(vault_config) => match verify_token(&client, &vault_config.api_token).await {
            Ok(_) => {
                let token = vault_config.api_token.clone();

                // renew
                let remaining_duration = compare_with_current_time(&vault_config.expiry_time);
                if remaining_duration < 1.hours() {
                    debug!("Extending the token expiration time");
                    let loader = new_spinner();
                    loader.set_message(
                        "Authenticating the user... You may need to check your device for an MFA notification.",
                    );

                    let renew_token_resp = renew_token(&client, &vault_config.api_token).await?;
                    let new_expiry_time =
                        calculate_expiry_time(renew_token_resp.auth.lease_duration);

                    config.vault = Some(VaultConfig {
                        api_token: token.clone(),
                        expiry_time: new_expiry_time,
                    });
                    config.save_to_default_path()?;

                    loader.finish_and_clear();
                }

                Ok(token)
            }
            Err(WKCliError::AuthError(AuthError::VaultPermissionDenied)) => {
                // login
                colored_println!("Login Vault with okta account {}", auth_config.account);

                let password = dialoguer::Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter okta password")
                    .interact()?;

                let loader = new_spinner();
                loader.set_message("Authenticating the user... You may need to check your device for an MFA notification.");

                let email = &auth_config.account;
                let login_resp = login(&client, email, &password).await?;

                loader.finish_and_clear();
                let expiry_time = calculate_expiry_time(login_resp.auth.lease_duration);

                config.vault = Some(VaultConfig {
                    api_token: login_resp.auth.client_token.clone(),
                    expiry_time,
                });
                config.save_to_default_path()?;

                colored_println!("You are now logged in as {}.\n", email);
                Ok(login_resp.auth.client_token)
            }
            Err(err) => Err(err),
        },
        None => {
            colored_println!("Login Vault with okta account {}", auth_config.account);

            let password = dialoguer::Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter okta password")
                .interact()?;

            let loader = new_spinner();
            loader.set_message(
            "Authenticating the user... You may need to check your device for an MFA notification.",
        );

            let email = &auth_config.account;
            let login_resp = login(&client, email, &password).await?;

            loader.finish_and_clear();
            let expiry_time = calculate_expiry_time(login_resp.auth.lease_duration);

            config.vault = Some(VaultConfig {
                api_token: login_resp.auth.client_token.clone(),
                expiry_time,
            });
            config.save_to_default_path()?;

            colored_println!("You are now logged in as {}.\n", email);
            Ok(login_resp.auth.client_token)
        }
    }
}

pub async fn login(
    client: &reqwest::Client,
    email: &str,
    password: &str,
) -> Result<Login, WKCliError> {
    debug!("Login user ...");
    let url = format!("{}{}/{}", BASE_URL.as_str(), LOGIN_URL, email);

    let response = client
        .post(url)
        .form(&[("password", password)])
        .send()
        .await?;

    debug!("login: {:?}", response);

    if !response.status().is_success() {
        let status = response.status();
        let message = response.text().await?;

        return Err(handle_error(status, message).await);
    }

    response.json::<Login>().await.map_err(|err| err.into())
}

pub async fn verify_token(client: &reqwest::Client, api_token: &str) -> Result<bool, WKCliError> {
    debug!("Verifying token ...");
    let url = format!("{}{}", BASE_URL.as_str(), VERIFY_TOKEN_URL);
    let loader = new_spinner();
    loader.set_message("Verifying the token...");

    let response = client
        .get(url)
        .header("X-Vault-Token", api_token)
        .send()
        .await?;

    loader.finish_and_clear();

    debug!("verify token: {:?}", response);

    if !response.status().is_success() {
        let status = response.status();
        let message = response.text().await?;

        return Err(handle_error(status, message).await);
    }

    Ok(true)
}

pub async fn renew_token(client: &reqwest::Client, api_token: &str) -> Result<Renew, WKCliError> {
    debug!("Renewing token ...");
    let url = format!("{}{}", BASE_URL.as_str(), RENEW_TOKEN_URL);

    let response = client
        .post(url)
        .header("X-Vault-Token", api_token)
        .form(&[("increment", "24h")])
        .send()
        .await?;

    debug!("renew token: {:?}", response);

    if !response.status().is_success() {
        let status = response.status();
        let message = response.text().await?;

        return Err(handle_error(status, message).await);
    }

    response.json::<Renew>().await.map_err(|err| err.into())
}

fn calculate_expiry_time(lease_duration: i64) -> String {
    let current_time: DateTime<Local> = Local::now();
    let expiry_time = current_time + Duration::seconds(lease_duration);

    expiry_time.to_rfc3339()
}

async fn handle_error(status: StatusCode, message: String) -> WKCliError {
    debug!("Vault Auth Error: status {status:?}, message: {message:?}");

    match status {
        StatusCode::NOT_FOUND => AuthError::VaultSecretNotFound.into(),
        StatusCode::FORBIDDEN => AuthError::VaultPermissionDenied.into(),
        StatusCode::BAD_REQUEST => {
            if message.contains("Okta auth failed") {
                AuthError::VaultAuthenticationFailed.into()
            } else {
                AuthError::VaultResponseError {
                    code: status.to_string(),
                    message,
                }
                .into()
            }
        }
        _ => AuthError::VaultResponseError {
            code: status.to_string(),
            message,
        }
        .into(),
    }
}

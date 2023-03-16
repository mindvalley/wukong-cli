use crate::config::VaultConfig;
use crate::output::colored_println;
use crate::{config::CONFIG_FILE, error::CliError, Config as CLIConfig};
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ResponseAuth {
    client_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    auth: ResponseAuth,
}

pub struct VaultClient {
    api_key: Option<String>,
}

impl VaultClient {
    pub fn new(api_key: Option<&str>) -> Self {
        let api_key_string = api_key.map(|s| s.to_string());

        Self {
            api_key: api_key_string,
        }
    }

    fn get_base_url(&self) -> Result<String, CliError> {
        let base_url = "https://bunker.mindvalley.dev:8200";
        let version = "v1";

        Ok(format!("{}/{}", base_url, version))
    }

    pub async fn handle_login(&self) -> Result<bool, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let config = CLIConfig::load(config_file)?;

        let mut email: Option<String> = None;

        if let Some(vault_config) = &config.vault {
            let selections = vec![
                "Use the current logged in account",
                "Log in with a new account",
            ];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                "You are already logged in as \"{}\", do you want to log in with a new account?",
                vault_config.email
            ))
                .default(0)
                .items(&selections[..])
                .interact()?;

            match selection {
                0 => {
                    colored_println!(
                        "You are already logged in as \"{}\", skipping login...",
                        vault_config.email
                    );

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

        if email.is_none() {
            let input_email: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Please enter your email address")
                .interact()?;

            email = Some(input_email.trim().to_string());
        }

        let password = dialoguer::Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your password")
            .interact()?;

        self.login(&email.unwrap().to_string(), password.as_str())
            .await?;

        Ok(true)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<bool, CliError> {
        debug!("Authenticating with the vault server ...");

        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let mut config = CLIConfig::load(config_file)?;

        let url = format!("{}/auth/okta/login/{}", self.get_base_url().unwrap(), email,);

        // FIXME: This is a hack
        info!("Check your phone - Waiting for the response from okta app");

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .form(&[("password", password)])
            .send()
            .await
            .map_err(|_err| CliError::UnAuthenticated)?
            .json::<ResponseData>()
            .await;

        config.vault = Some(VaultConfig {
            api_key: response.as_ref().unwrap().auth.client_token.to_string(),
            email: email.to_string(),
        });

        config.save(config_file).unwrap();

        colored_println!("You are now logged in as {}.", email.to_string());

        Ok(true)
    }

    async fn get_api_key(&self) -> Result<String, CliError> {
        let api_token = self.api_key.clone();

        if api_token.is_none() {}

        Ok(api_token.unwrap())
    }

    async fn validate_user(&self) -> Result<bool, CliError> {
        let token = self.get_api_key().await?;

        // TODO: If the token is valid, return true
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

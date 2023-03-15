use crate::{
    commands::Context,
    config::CONFIG_FILE,
    services::vault::client::VaultClient,
    telemetry::{self, TelemetryData, TelemetryEvent},
    CliError, Config as CLIConfig,
};
use dialoguer::{theme::ColorfulTheme, Select};
use wukong_telemetry_macro::wukong_telemetry;

#[wukong_telemetry(command_event = "vault_login")]
pub async fn handle_login(context: Context) -> Result<bool, CliError> {
    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");

    let config = CLIConfig::load(config_file)?;

    if let Some(auth_config) = &config.auth {
        let selections = vec![
            "Use the current logged in account",
            "Log in with a new account",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "You are already logged in as \"{}\", do you want to log in with a new account?",
                auth_config.account
            ))
            .default(0)
            .items(&selections[..])
            .interact()?;

        // selecting "Log in with a new account"
        if selection == 1 {
            VaultClient::new(None).login(config).await?;
        }
    } else {
        VaultClient::new(None).login(config).await?;
    }

    Ok(true)
}

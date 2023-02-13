use crate::{
    app::APP_CONFIG,
    auth::Auth,
    config::{AuthConfig, CONFIG_FILE},
    error::CliError,
    output::colored_println,
    Config as CLIConfig,
};
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn handle_login() -> Result<bool, CliError> {
    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");

    let config = CLIConfig::load(config_file)?;

    let current_config = config.clone();

    APP_CONFIG.set(config).unwrap();

    if let Some(auth_config) = current_config.auth {
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
            login_and_update_config().await?;
        }
    } else {
        login_and_update_config().await?;
    }

    Ok(true)
}

async fn login_and_update_config() -> Result<bool, CliError> {
    let auth_info = Auth::new(&APP_CONFIG.get().unwrap().core.okta_client_id)
        .login()
        .await?;

    let mut current_config = APP_CONFIG.get().unwrap().clone();
    current_config.auth = Some(AuthConfig {
        account: auth_info.account.clone(),
        subject: auth_info.subject.clone(),
        id_token: auth_info.id_token,
        access_token: auth_info.access_token,
        expiry_time: auth_info.expiry_time,
        refresh_token: auth_info.refresh_token,
    });

    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");

    current_config.save(config_file).unwrap();
    colored_println!("You are now logged in as {}.", auth_info.account);

    Ok(true)
}

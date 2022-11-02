use crate::{
    auth,
    config::{AuthConfig, CONFIG_FILE},
    error::CliError,
    Config as CLIConfig, GlobalContext,
};
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn handle_login(context: GlobalContext) -> Result<bool, CliError> {
    if let Some(account) = context.account {
        let selections = vec![
            "Use the current logged in account",
            "Log in with a new account",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "You are already logged in as \"{}\", do you want to log in with a new account?",
                account
            ))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

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
    let auth_info = auth::login().await?;

    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");

    CLIConfig::load(config_file).map(|mut config| {
        config.auth = Some(AuthConfig {
            account: auth_info.account.clone(),
            id_token: auth_info.id_token,
            access_token: auth_info.access_token,
            expiry_time: auth_info.expiry_time,
            refresh_token: auth_info.refresh_token,
        });
        config.save(config_file).unwrap();
        println!("You are now logged in as [{}].", auth_info.account);
    })?;

    Ok(true)
}

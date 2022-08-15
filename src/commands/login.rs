use dialoguer::{theme::ColorfulTheme, Select};

use crate::{
    auth,
    config::{AuthConfig, CONFIG_FILE},
    error::CliError,
    Config as CLIConfig, GlobalContext,
};

pub async fn handle_login<'a>(context: GlobalContext) -> Result<bool, CliError<'a>> {
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

        if selection == 1 {
            let auth_info = auth::login().await?;

            let config_file = CONFIG_FILE
                .as_ref()
                .expect("Unable to identify user's home directory");

            match CLIConfig::load(&config_file) {
                Ok(mut config) => {
                    config.auth = Some(AuthConfig {
                        account: auth_info.account.clone(),
                        access_token: auth_info.access_token,
                        expiry_time: auth_info.expiry_time,
                        refresh_token: auth_info.refresh_token,
                    });
                    config.save(&config_file).unwrap();
                    println!("You are now logged in as [{}].", auth_info.account);
                }
                Err(_err) => todo!(),
            };
        }
    }

    Ok(true)
}

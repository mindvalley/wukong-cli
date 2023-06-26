use crate::{
    auth::Auth,
    config::AuthConfig,
    error::{AuthError, CliError},
    loader::new_spinner_progress_bar,
    output::colored_println,
    Config as CLIConfig,
};
use aion::*;
use chrono::{DateTime, Local};
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;
use openidconnect::RefreshToken;

pub async fn handle_login() -> Result<bool, CliError> {
    let mut config = CLIConfig::load_from_default_path()?;

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
            login_and_update_config(config).await?;
        } else {
            // checking the refresh token
            let auth_config = config.auth.as_ref().unwrap();

            let current_time: DateTime<Local> = Local::now();
            let expiry = DateTime::parse_from_rfc3339(&auth_config.expiry_time)
                .unwrap()
                .with_timezone(&Local);
            let remaining_duration = expiry.signed_duration_since(current_time);

            if remaining_duration < 5.minutes() {
                debug!("Access token expired. Refreshing tokens...");

                let refresh_token_loader = new_spinner_progress_bar();
                refresh_token_loader.set_message("Refreshing tokens...");

                match Auth::new(&config.core.okta_client_id)
                    .refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone()))
                    .await
                {
                    Ok(new_tokens) => {
                        config.auth = Some(AuthConfig {
                            account: auth_config.account.clone(),
                            subject: auth_config.subject.clone(),
                            id_token: new_tokens.id_token.clone(),
                            access_token: new_tokens.access_token.clone(),
                            expiry_time: new_tokens.expiry_time,
                            refresh_token: new_tokens.refresh_token,
                        });

                        refresh_token_loader.finish_and_clear();
                    }
                    Err(err) => {
                        refresh_token_loader.finish_and_clear();
                        match err {
                            AuthError::RefreshTokenExpired { .. } => {
                                eprintln!("The refresh token is expired. You have to login again.");
                                login_and_update_config(config).await?;
                            }
                            err => return Err(err.into()),
                        }
                    }
                }
            }
        }
    } else {
        login_and_update_config(config).await?;
    }

    Ok(true)
}

async fn login_and_update_config(mut current_config: CLIConfig) -> Result<bool, CliError> {
    let auth_info = Auth::new(&current_config.core.okta_client_id)
        .login()
        .await?;

    current_config.auth = Some(AuthConfig {
        account: auth_info.account.clone(),
        subject: auth_info.subject.clone(),
        id_token: auth_info.id_token,
        access_token: auth_info.access_token,
        expiry_time: auth_info.expiry_time,
        refresh_token: auth_info.refresh_token,
    });

    current_config.save_to_default_path()?;
    colored_println!("You are now logged in as {}.", auth_info.account);

    Ok(true)
}

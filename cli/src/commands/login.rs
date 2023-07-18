use crate::{
    config::{AuthConfig, Config},
    error::WKCliError,
    loader::new_spinner,
    output::colored_println,
    utils::compare_with_current_time,
};
use aion::*;
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;
use wukong_sdk::{error::AuthError, OktaAuthenticator};

pub async fn handle_login() -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;

    let mut login_selections = vec!["Log in with a new account"];
    if let Some(ref auth_config) = config.auth {
        login_selections.splice(..0, vec![auth_config.account.as_str()]);
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose the account you would like to use to perform operations for this configuration:")
                .default(0)
                .items(&login_selections[..])
                .interact()?;

    // "Log in with a new account" is selected
    let new_config = if selection == login_selections.len() - 1 {
        login_and_create_config(Config::default()).await?
    } else {
        // check access token expiry
        let mut current_config = config.clone();
        let auth_config = current_config.auth.as_ref().unwrap();

        let remaining_duration = compare_with_current_time(&auth_config.expiry_time);
        if remaining_duration < 5.minutes() {
            debug!("Access token expired. Refreshing tokens...");

            let refresh_token_loader = new_spinner();
            refresh_token_loader.set_message("Refreshing tokens...");

            let okta_authenticator = OktaAuthenticator::builder()
                .with_okta_id(&config.core.okta_client_id)
                .with_callback_url("http://localhost:6758/login/callback")
                .build();

            let updated_config = match okta_authenticator
                .refresh_tokens(auth_config.refresh_token.clone())
                .await
            {
                Ok(new_tokens) => {
                    current_config.auth = Some(AuthConfig {
                        account: auth_config.account.clone(),
                        subject: auth_config.subject.clone(),
                        id_token: new_tokens.id_token.clone(),
                        access_token: new_tokens.access_token.clone(),
                        expiry_time: new_tokens.expiry_time,
                        refresh_token: new_tokens.refresh_token,
                    });

                    refresh_token_loader.finish_and_clear();
                    colored_println!("You are logged in as: {}.\n", login_selections[selection]);

                    current_config
                }
                Err(err) => {
                    refresh_token_loader.finish_and_clear();
                    match err {
                        AuthError::RefreshTokenExpired { .. } => {
                            eprintln!("The refresh token is expired. You have to login again.");
                            login_and_create_config(current_config).await?
                        }
                        err => return Err(err.into()),
                    }
                }
            };

            current_config = updated_config;
        } else {
            colored_println!("You are logged in as: {}.\n", login_selections[selection]);
        }

        current_config
    };

    new_config.save_to_default_path()?;

    Ok(true)
}

async fn login_and_create_config(mut config: Config) -> Result<Config, WKCliError> {
    let okta_authenticator = OktaAuthenticator::builder()
        .with_okta_id(&config.core.okta_client_id)
        .with_callback_url("http://localhost:6758/login/callback")
        .build();
    let auth_info = okta_authenticator.login().await?;

    // we don't really care about the exisiting config (id_token, refresh_token, account)
    // if the user choose to log in with a new account
    config.auth = Some(AuthConfig {
        account: auth_info.account.clone(),
        subject: auth_info.subject.clone(),
        id_token: auth_info.id_token,
        access_token: auth_info.access_token,
        expiry_time: auth_info.expiry_time,
        refresh_token: auth_info.refresh_token,
    });

    colored_println!("You are logged in as: {}.\n", auth_info.account);

    Ok(config)
}

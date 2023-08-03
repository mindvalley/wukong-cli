use crate::{
    auth,
    config::Config,
    error::{AuthError, WKCliError},
    loader::new_spinner,
    output::colored_println,
};
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;

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
        login_and_create_config(config).await?
    } else {
        // check access token expiry
        let mut current_config = config.clone();
        if auth::okta::need_tokens_refresh(&config)? {
            debug!("Access token expired. Refreshing tokens...");

            let refresh_token_loader = new_spinner();
            refresh_token_loader.set_message("Refreshing tokens...");

            let updated_config = match auth::okta::refresh_tokens(&config).await {
                Ok(new_tokens) => {
                    current_config.auth = Some(new_tokens.into());

                    refresh_token_loader.finish_and_clear();
                    colored_println!("You are logged in as: {}.\n", login_selections[selection]);

                    current_config
                }
                Err(err) => {
                    refresh_token_loader.finish_and_clear();
                    match err {
                        WKCliError::AuthError(AuthError::OktaRefreshTokenExpired { .. }) => {
                            eprintln!("The refresh token is expired. You have to login again.");
                            login_and_create_config(current_config).await?
                        }
                        err => return Err(err),
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
    let auth_info = auth::okta::login(&config).await?;
    let acc = auth_info.account.clone();

    config.auth = Some(auth_info.into());

    colored_println!("You are logged in as: {acc}.\n");

    Ok(config)
}

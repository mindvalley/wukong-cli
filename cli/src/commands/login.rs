use crate::{
    auth,
    config::Config,
    error::{AuthError, WKCliError},
    loader::new_spinner,
    output::colored_println,
    utils::inquire::inquire_render_config,
};
use crossterm::style::Stylize;
use log::debug;

pub async fn handle_login() -> Result<bool, WKCliError> {
    let config = Config::load_from_default_path()?;

    let new_config = new_login_or_refresh_token(config).await?;
    new_config.save_to_default_path()?;

    Ok(true)
}

pub async fn new_login_or_refresh_token(config: Config) -> Result<Config, WKCliError> {
    let mut login_selections = vec!["Login with a new account"];
    if let Some(ref okta_config) = config.auth.okta {
        login_selections.splice(..0, vec![okta_config.account.as_str()]);
    };

    let selected_account = inquire::Select::new(
        "Choose the account you would like to continue with",
        login_selections,
    )
    .with_render_config(inquire_render_config())
    .prompt()?;

    // "Log in with a new account" is selected
    let new_config = if selected_account == "Login with a new account" {
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
                    current_config.auth.okta = Some(new_tokens.into());

                    refresh_token_loader.finish_and_clear();
                    colored_println!("You are logged in as: {}", selected_account.dark_cyan());

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
            colored_println!("You are logged in as: {}", selected_account.dark_cyan());
        }

        current_config
    };

    Ok(new_config)
}

async fn login_and_create_config(mut config: Config) -> Result<Config, WKCliError> {
    let auth_info = auth::okta::login().await?;
    let acc = auth_info.account.clone();

    config.auth.okta = Some(auth_info.into());

    colored_println!("You are logged in as: {}", acc.dark_cyan());

    Ok(config)
}

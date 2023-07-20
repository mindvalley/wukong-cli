use crate::{
    auth::Auth,
    config::{AuthConfig, Config},
    error::{AuthError, ConfigError, WKError},
    graphql::QueryClient,
    loader::new_spinner_progress_bar,
    output::colored_println,
};
use aion::*;
use chrono::{DateTime, Local};
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;
use openidconnect::RefreshToken;

pub async fn handle_init() -> Result<bool, WKError> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");

    let config = match Config::load_from_default_path() {
        Ok(config) => config,
        Err(error) => match error {
            ConfigError::NotFound { .. } | ConfigError::BadTomlData(_) => Config::default(),
            error => return Err(error.into()),
        },
    };

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
    let mut new_config = if selection == login_selections.len() - 1 {
        let config = Config::default();
        login_and_create_config(config).await?
    } else {
        // check access token expiry
        let mut current_config = config.clone();
        let auth_config = current_config.auth.as_ref().unwrap();

        let current_time: DateTime<Local> = Local::now();
        let expiry = DateTime::parse_from_rfc3339(&auth_config.expiry_time)
            .unwrap()
            .with_timezone(&Local);
        let remaining_duration = expiry.signed_duration_since(current_time);

        if remaining_duration < 5.minutes() {
            debug!("Access token expired. Refreshing tokens...");

            let refresh_token_loader = new_spinner_progress_bar();
            refresh_token_loader.set_message("Refreshing tokens...");

            let updated_config = match Auth::new(&config.core.okta_client_id)
                .refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone()))
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

    let fetch_loader = new_spinner_progress_bar();
    fetch_loader.set_message("Fetching application list...");
    // Calling API ...
    let mut client = QueryClient::from_config(&new_config)?;

    let applications_data: Vec<String> = client
        .fetch_application_list()
        .await?
        .data
        .unwrap()
        .applications
        .iter()
        .map(|application| application.name.clone())
        .collect();
    fetch_loader.finish_and_clear();

    let application_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please select the application")
        .default(0)
        .items(&applications_data[..])
        .interact()?;

    colored_println!(
        "Your current application has been set to: {}.",
        &applications_data[application_selection]
    );

    new_config.core.application = applications_data[application_selection].clone();

    colored_println!(
        r#"
Your Wukong CLI is configured and ready to use!

* Commands that require authentication will use {} by default
* Commands will reference application {} by default
Run `wukong config help` to learn how to change individual settings

Some things to try next:

* Run `wukong --help` to see the wukong command groups you can interact with. And run `wukong COMMAND help` to get help on any wukong command.
                     "#,
        new_config.auth.as_ref().unwrap().account,
        new_config.core.application
    );

    new_config.save_to_default_path()?;

    Ok(true)
}

async fn login_and_create_config(mut config: Config) -> Result<Config, WKError> {
    let auth_info = Auth::new(&config.core.okta_client_id).login().await?;

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
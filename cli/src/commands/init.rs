use crate::{
    auth,
    config::Config,
    error::{AuthError, ConfigError, WKCliError},
    loader::new_spinner,
    output::colored_println,
    utils::wukong_sdk::FromWKCliConfig,
};
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;
use wukong_sdk::WKClient;

pub async fn handle_init() -> Result<bool, WKCliError> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");

    let config = match Config::load_from_default_path() {
        Ok(config) => config,
        Err(error) => match error {
            // create new config if the config file not found or the config format is invalid
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
        login_and_create_config(Config::default()).await?
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

                    current_config
                }
                Err(err) => {
                    refresh_token_loader.finish_and_clear();
                    match err {
                        WKCliError::AuthError(AuthError::OktaRefreshTokenExpired { .. }) => {
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

    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching application list...");

    let wk_client = WKClient::from_cli_config(&new_config);

    let applications_data: Vec<String> = wk_client
        .fetch_applications()
        .await?
        .applications
        .iter()
        .map(|app| app.name.clone())
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

    new_config
        .save_to_default_path()
        .expect("Config file save failed");

    Ok(true)
}

async fn login_and_create_config(mut config: Config) -> Result<Config, WKCliError> {
    let auth_info = auth::okta::login(&config).await?;
    let acc = auth_info.account.clone();

    config.auth = Some(auth_info.into());

    colored_println!("You are logged in as: {acc}.\n");

    Ok(config)
}

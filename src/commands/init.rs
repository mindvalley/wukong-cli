use crate::{
    auth::Auth,
    config::{AuthConfig, Config, CONFIG_FILE},
    error::{CliError, ConfigError},
    graphql::QueryClientBuilder,
    output::colored_println,
};
use chrono::{DateTime, Local};
use dialoguer::{theme::ColorfulTheme, Select};
use log::debug;
use openidconnect::RefreshToken;

pub async fn handle_init() -> Result<bool, CliError> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");

    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");

    let config = match Config::load(config_file) {
        Ok(config) => config,
        Err(error) => match error {
            CliError::ConfigError(ConfigError::NotFound { .. })
            | CliError::ConfigError(ConfigError::BadTomlData(_)) => Config::default(),
            error => return Err(error),
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
        let mut config = Config::default();

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
        config
    } else {
        // check access token expiry
        let mut current_config = config.clone();
        let auth_config = current_config.auth.as_ref().unwrap();

        let local: DateTime<Local> = Local::now();
        let expiry = DateTime::parse_from_rfc3339(&auth_config.expiry_time)
            .unwrap()
            .with_timezone(&Local);

        if local >= expiry {
            debug!("Access token expired. Refreshing tokens...");

            let new_tokens = Auth::new(&config.core.okta_client_id)
                .refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone()))
                .await?;

            current_config.auth = Some(AuthConfig {
                account: auth_config.account.clone(),
                subject: auth_config.subject.clone(),
                id_token: new_tokens.id_token.clone(),
                access_token: new_tokens.access_token.clone(),
                expiry_time: new_tokens.expiry_time,
                refresh_token: new_tokens.refresh_token,
            });
        }

        colored_println!("You are logged in as: {}.\n", login_selections[selection]);

        current_config
    };

    // SAFETY: The auth must not be None here
    let auth_config = new_config.auth.as_ref().unwrap();

    // Calling API ...
    let client = QueryClientBuilder::default()
        .with_access_token(auth_config.id_token.clone())
        .with_sub(Some(auth_config.subject.clone()))
        .with_api_url(new_config.core.wukong_api_url.clone())
        .build()?;

    let applications_data: Vec<String> = client
        .fetch_application_list()
        .await?
        .data
        .unwrap()
        .applications
        .iter()
        .map(|application| application.name.clone())
        .collect();

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

    if let Some(ref config_file) = *CONFIG_FILE {
        new_config
            .save(config_file)
            .expect("Config file save failed");
    }

    Ok(true)
}

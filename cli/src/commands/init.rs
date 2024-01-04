use crate::{
    auth::vault,
    commands::google,
    commands::login,
    config::{ApiChannel, Config},
    error::{ConfigError, WKCliError},
    loader::new_spinner,
    output::colored_println,
    wukong_client::WKClient,
};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use owo_colors::OwoColorize;

pub async fn handle_init(channel: ApiChannel) -> Result<bool, WKCliError> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");

    let config = match Config::load_from_default_path() {
        Ok(config) => config,
        Err(error) => match error {
            // create new config if the config file not found or the config format is invalid
            ConfigError::NotFound { .. } | ConfigError::BadTomlData(_) => Config::default(),
            error => return Err(error.into()),
        },
    };

    login::handle_login(Some(config)).await?;

    let mut new_config = Config::load_from_default_path()?;
    new_config = handle_application(new_config, channel).await?;
    new_config = handle_gcloud_auth(new_config).await?;
    new_config = handle_bunker_auth(new_config).await?;

    colored_println!(
        r#"
    Your Wukong CLI is configured and ready to use!

    * Commands that require authentication will use {} by default
    * Commands will reference application {} by default
    Run `wukong config help` to learn how to change individual settings

    Some things to try next:

    * Run `wukong --help` to see the wukong command groups you can interact with. And run `wukong COMMAND help` to get help on any wukong command.
                             "#,
        new_config.auth.okta.as_ref().unwrap().account,
        new_config.core.application
    );

    new_config
        .save_to_default_path()
        .expect("Config file save failed");

    Ok(true)
}

async fn handle_bunker_auth(mut config: Config) -> Result<Config, WKCliError> {
    let agree_to_authenticate = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Do you want to authenticate against Bunker? You may do it later when neccessary"
        ))
        .default(true)
        .interact()?;

    if agree_to_authenticate {
        vault::get_token_or_login(&mut config).await?;
    }

    Ok(config)
}

async fn handle_gcloud_auth(new_config: Config) -> Result<Config, WKCliError> {
    let agree_to_authenticate = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Do you want to authenticate against Google Cloud? You may do it later when neccessary"
        ))
        .default(false)
        .interact()?;

    if agree_to_authenticate {
        google::login::handle_login().await?;
    }

    Ok(new_config)
}

async fn handle_application(mut config: Config, channel: ApiChannel) -> Result<Config, WKCliError> {
    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching application list...");

    let mut wk_client = WKClient::for_channel(&config, &channel)?;

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

    config.core.application = applications_data[application_selection].clone();

    Ok(config)
}

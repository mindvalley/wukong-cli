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
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;

pub static TMP_CONFIG_FILE: Lazy<Option<String>> = Lazy::new(|| {
    return dirs::home_dir().map(|mut path| {
        path.extend([".config", "wukong", ".tmp", "config.toml"]);
        path.to_str().unwrap().to_string()
    });
});

pub async fn handle_init(channel: ApiChannel) -> Result<bool, WKCliError> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");
    let mut config = match Config::load_from_default_path() {
        Ok(config) => config,
        Err(error) => match error {
            // create new config if the config file not found or the config format is invalid
            ConfigError::NotFound { .. } | ConfigError::BadTomlData(_) => Config::default(),
            error => return Err(error.into()),
        },
    };

    config = login::new_login_or_refresh_token(config).await?;
    config = handle_application(config, channel).await?;
    config = handle_gcloud_auth(config).await?;
    config = handle_vault_auth(config).await?;

    config
        .save_to_default_path()
        .expect("Config file save failed");

    colored_println!(
        r#"
    Your Wukong CLI is configured and ready to use!

    * Commands that require authentication will use {} by default
    * Commands will reference application {} by default
    Run `wukong config help` to learn how to change individual settings

    Some things to try next:

    * Run `wukong --help` to see the wukong command groups you can interact with. And run `wukong COMMAND help` to get help on any wukong command.
                             "#,
        config.auth.okta.as_ref().unwrap().account,
        config.core.application
    );

    Ok(true)
}

async fn handle_vault_auth(mut config: Config) -> Result<Config, WKCliError> {
    let agree_to_authenticate = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Do you want to authenticate against Bunker? You may do it later when neccessary"
        ))
        .default(false)
        .interact()?;

    if agree_to_authenticate {
        vault::get_token_or_login(&mut config).await?;
    }

    Ok(config)
}

async fn handle_gcloud_auth(mut config: Config) -> Result<Config, WKCliError> {
    let agree_to_authenticate = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} {}",
            "(Optional)".bright_black(),
            "Do you want to authenticate against Google Cloud? You may do it later when neccessary"
        ))
        .default(false)
        .interact()?;

    if agree_to_authenticate {
        // Use temporary file to achieve atomic write for gcloud:
        // GCloud does not support atomic write, so we use a temporary file to store the config
        // and then move it to the original location after when the config is ready.
        let tmp_config = Config::default()
            .with_path(TMP_CONFIG_FILE.to_owned().expect("Unable to get tmp path"));

        google::login::handle_login(Some(tmp_config.clone())).await?;

        // Load the config again to get the latest token
        let updated_config =
            Config::load_from_path(TMP_CONFIG_FILE.as_ref().expect("Unable to get tmp path"))
                .expect("Unable to load tmp config");

        config.auth.google_cloud = updated_config.auth.google_cloud.clone();

        tmp_config
            .remove_config_from_path()
            .expect("Config file failed to remove");

        return Ok(config);
    }

    Ok(config)
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

use crate::{
    auth::vault,
    commands::{google, login},
    config::Config,
    error::{ConfigError, WKCliError},
    output::colored_println,
    utils::inquire::inquire_render_config,
};
use crossterm::style::Stylize;
use once_cell::sync::Lazy;

pub static TMP_CONFIG_FILE: Lazy<Option<String>> = Lazy::new(|| {
    return dirs::home_dir().map(|mut path| {
        path.extend([".config", "wukong", ".tmp", "config.toml"]);
        path.to_str().unwrap().to_string()
    });
});

pub async fn handle_init() -> Result<bool, WKCliError> {
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
    config = handle_gcloud_auth(config).await?;
    config = handle_vault_auth(config).await?;

    config
        .save_to_default_path()
        .expect("Config file save failed");

    colored_println!(
        r#"
{}

{} Commands that require authentication will use {} by default
{} Run `wukong config help` to learn how to change individual settings

{}
{} Run `wukong --help` to see the wukong command groups you can interact with. And run `wukong COMMAND help` to get help on any wukong command.
                             "#,
        "Your Wukong CLI is configured and ready to use!".bold(),
        "▸".green(),
        config
            .auth
            .okta
            .expect("Okta account not found")
            .account
            .dark_cyan(),
        "▸".green(),
        "Some things to try next:".bold(),
        "▸".green(),
    );

    Ok(true)
}

async fn handle_vault_auth(mut config: Config) -> Result<Config, WKCliError> {
    let agree_to_authenticate =
        inquire::Confirm::new("Do you want to authenticate against Bunker?")
            .with_render_config(inquire_render_config())
            .with_help_message("You may able to login later when neccessary")
            .with_default(false)
            .prompt()?;

    if agree_to_authenticate {
        vault::get_token_or_login(&mut config).await?;
    }

    Ok(config)
}

async fn handle_gcloud_auth(mut config: Config) -> Result<Config, WKCliError> {
    let agree_to_authenticate =
        inquire::Confirm::new("Do you want to authenticate against Google Cloud?")
            .with_render_config(inquire_render_config())
            .with_help_message("You may able to login later when neccessary")
            .with_default(false)
            .prompt()?;

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

use crate::{
    commands::login::handle_login,
    config::{ApiChannel, Config},
    error::{ConfigError, WKCliError},
    loader::new_spinner,
    output::colored_println,
    wukong_client::WKClient,
};
use dialoguer::{theme::ColorfulTheme, Select};

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

    handle_login(Some(config)).await?;

    let mut new_config = Config::load_from_default_path()?;

    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching application list...");

    let mut wk_client = WKClient::for_channel(&new_config, &channel)?;

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

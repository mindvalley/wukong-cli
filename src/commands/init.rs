use crate::{
    auth::login,
    config::{AuthConfig, Config, CONFIG_FILE},
    error::CliError,
    graphql::QueryClientBuilder,
    GlobalContext,
};
use dialoguer::{theme::ColorfulTheme, Select};

pub async fn handle_init<'a>(
    context: GlobalContext,
    existing_config: Option<Config>,
) -> Result<bool, CliError<'a>> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");

    let mut login_selections = vec!["Log in with a new account"];
    if let Some(ref account) = context.account {
        login_selections.splice(..0, vec![account.as_str()]);
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose the account you would like to use to perform operations for this configuration:")
                .default(0)
                .items(&login_selections[..])
                .interact()
                .unwrap();

    let mut config = match existing_config {
        Some(ref config) => config.clone(),
        None => Config::default(),
    };

    // "Log in with a new account" is selected
    if selection == login_selections.len() - 1 {
        let auth_info = login().await?;

        config.auth = Some(AuthConfig {
            account: auth_info.account.clone(),
            access_token: auth_info.access_token,
            expiry_time: auth_info.expiry_time,
            refresh_token: auth_info.refresh_token,
        });

        println!("You are logged in as: [{}].\n", auth_info.account);
    } else {
        println!("You are logged in as: [{}].\n", login_selections[selection]);
    }

    // Calling API ...
    let client = QueryClientBuilder::new()
        .with_access_token(context.access_token.unwrap())
        .build()?;

    let applications_data: Vec<String> = client
        .fetch_application_list()
        .await?
        .data
        .unwrap()
        .applications
        .expect("Application list can't be empty.")
        .iter()
        .filter(|application| application.is_some())
        .map(|application| {
            // unwrap is safe here because we are already filtered out None in previous step
            application.as_ref().unwrap().name.clone()
        })
        .collect();

    let application_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please select the application")
        .default(0)
        .items(&applications_data[..])
        .interact()
        .unwrap();

    println!(
        "Your current application has been set to: [{}].",
        &applications_data[application_selection]
    );

    config.core.application = applications_data[application_selection].clone();

    println!(
        r#"
Your Wukong CLI is configured and ready to use!

* Commands that require authentication will use junkai.gan@mindvalley.com by default
* Commands will reference application `mv-prod-applications-hub` by default
Run `wukong config help` to learn how to change individual settings

Some things to try next:

* Run `wukong --help` to see the wukong command groups you can interact with. And run `wukong COMMAND help` to get help on any wukong command.
                     "#
    );

    if let Some(ref config_file) = *CONFIG_FILE {
        config.save(config_file).expect("Config file save failed");
    }

    Ok(true)
}

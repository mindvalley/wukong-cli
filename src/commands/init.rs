use dialoguer::{theme::ColorfulTheme, Select};

use crate::{
    config::{Config, CONFIG_FILE},
    error::CliError,
    GlobalContext,
};

pub fn handle_init<'a>(_context: GlobalContext) -> Result<bool, CliError<'a>> {
    println!("Welcome! This command will take you through the configuration of Wukong.\n");

    let login_selections = &["junkai.gan@mindvalley.com", "Log in with a new account"];

    let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose the account you would like to use to perform operations for this configuration:")
                .default(0)
                .items(&login_selections[..])
                .interact()
                .unwrap();

    println!("You are logged in as: [{}].\n", login_selections[selection]);

    let application_selections = &[
        "mv-prod-applications-hub",
        "mv-prod-linode",
        "mv-prod-platform-osiris",
        "mv-stg-applications-hub",
        "mv-stg-dev-platform-osiris",
        "mv-stg-linode",
    ];

    let application_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Please select the application")
        .default(0)
        .items(&application_selections[..])
        .interact()
        .unwrap();

    println!(
        "Your current application has been set to: [{}].",
        &application_selections[application_selection]
    );

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
        let mut config = Config::default();
        config.core.application = application_selections[application_selection].to_string();
        config.save(config_file).expect("Config file save failed");
    }

    Ok(true)
}

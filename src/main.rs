#![forbid(unsafe_code)]

mod app;
mod auth;
mod clap_app;
mod command_group;
mod config;
mod error;
mod graphql;
// mod logger;

use command_group::CommandGroup;
use config::{Config, CONFIG_FILE};
use dialoguer::{theme::ColorfulTheme, Select};
use error::{handle_error, CliError};
// use logger::Logger;
use app::App;
use std::process;

pub struct GlobalContext {
    application: Option<String>,

    // auth
    access_token: Option<String>,
    expiry_time: Option<String>,
    refresh_token: Option<String>,
}

#[tokio::main]
async fn main() {
    // Logger::new().init();

    // auth::login().await;
    let result = run();

    match result.await {
        Err(error) => {
            handle_error(error);
            process::exit(1);
        }
        Ok(false) => {
            process::exit(1);
        }
        Ok(true) => {
            process::exit(0);
        }
    }
}

async fn run<'a>() -> Result<bool, CliError<'a>> {
    let app = App::new()?;

    let current_application = {
        if let Some(ref application) = app.cli.application {
            Some(application.clone())
        } else {
            match app.config {
                app::ConfigState::Initialized(ref config) => Some(config.core.application.clone()),
                app::ConfigState::Uninitialized => None,
            }
        }
    };

    let mut context = GlobalContext {
        application: current_application,
        access_token: None,
        expiry_time: None,
        refresh_token: None,
    };

    if let app::ConfigState::Initialized(config) = app.config {
        if let Some(auth_config) = &config.auth {
            context.access_token = Some(auth_config.access_token.clone());
            context.refresh_token = Some(auth_config.refresh_token.clone());
            context.expiry_time = Some(auth_config.expiry_time.clone());
        }
    }

    match app.cli.command_group {
        CommandGroup::Pipeline(pipeline) => pipeline.perform_action(context).await,
        CommandGroup::Config(config) => config.perform_action(),
        CommandGroup::Init => {
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
        CommandGroup::Login => {
            auth::login().await;
            Ok(true)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::clap_app::ClapApp;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;

        ClapApp::command().debug_assert()
    }
}

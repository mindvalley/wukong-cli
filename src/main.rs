#![forbid(unsafe_code)]

mod command_group;
mod config;
mod error;
mod graphql;
// mod logger;

use anyhow::Result as AnyhowResult;
use clap::Parser;
use command_group::CommandGroup;
use config::{Config, CONFIG_FILE};
use dialoguer::{theme::ColorfulTheme, Select};
use error::{handle_error, CliError};
// use logger::Logger;
use std::process;

/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Debug, Parser)]
#[clap(version, author)]
struct Cli {
    #[clap(subcommand)]
    command_group: CommandGroup,

    /// Override the application name that the CLI will perform the command against.
    /// If the flag is not used, then the CLI will use the default application name from the config.
    #[clap(long, short)]
    application: Option<String>,
}

#[tokio::main]
async fn main() {
    // Logger::new().init();

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

async fn run<'a>() -> AnyhowResult<bool, CliError<'a>> {
    let cli = Cli::parse();

    match cli.command_group {
        CommandGroup::Pipeline(pipeline) => pipeline.perform_action().await?,
        CommandGroup::Config(config) => {
            config.perform_action();
        }
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
        }
    }

    Ok(true)
}

#[cfg(test)]
mod test {
    use crate::Cli;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;

        Cli::command().debug_assert()
    }
}

#![forbid(unsafe_code)]

mod app;
mod auth;
mod clap_app;
mod commands;
mod config;
mod error;
mod graphql;
// mod logger;

use commands::{init::handle_init, login::handle_login, CommandGroup};
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
        CommandGroup::Config(config) => config.perform_action(context),
        CommandGroup::Init => handle_init(context),
        CommandGroup::Login => handle_login(context).await,
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

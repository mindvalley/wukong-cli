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
use error::{handle_error, CliError};
// use logger::Logger;
use app::{App, ConfigState};
use std::process;

#[derive(Default)]
pub struct GlobalContext {
    application: Option<String>,
    account: Option<String>,
    access_token: Option<String>,
}

#[tokio::main]
async fn main() {
    // Logger::new().init();

    // auth::login().await;
    match run().await {
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

    let mut context = GlobalContext::default();

    match app.config {
        app::ConfigState::InitialisedAndAuthenticated(ref config) => {
            context.application = Some(config.core.application.clone());
            context.account = Some(config.auth.as_ref().unwrap().account.clone());
            context.access_token = Some(config.auth.as_ref().unwrap().access_token.clone());
        }
        app::ConfigState::InitialisedButUnAuthenticated(ref config) => {
            context.application = Some(config.core.application.clone());
        }
        app::ConfigState::Uninitialised => {}
    };

    // overwritten by --application flag
    if let Some(ref application) = app.cli.application {
        context.application = Some(application.clone());
    }

    match app.cli.command_group {
        CommandGroup::Pipeline(pipeline) => match app.config {
            ConfigState::InitialisedAndAuthenticated(_) => pipeline.perform_action(context).await,
            ConfigState::InitialisedButUnAuthenticated(_) => return Err(CliError::UnAuthenticated),
            ConfigState::Uninitialised => return Err(CliError::UnInitialised),
        },
        CommandGroup::Config(config) => match app.config {
            ConfigState::InitialisedAndAuthenticated(_)
            | ConfigState::InitialisedButUnAuthenticated(_) => config.perform_action(context),
            ConfigState::Uninitialised => return Err(CliError::UnInitialised),
        },
        CommandGroup::Init => handle_init(context).await,
        CommandGroup::Login => match app.config {
            ConfigState::InitialisedAndAuthenticated(_)
            | ConfigState::InitialisedButUnAuthenticated(_) => handle_login(context).await,
            ConfigState::Uninitialised => return Err(CliError::UnInitialised),
        },
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

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

macro_rules! must_init {
    ($config:expr, $instance:ident.$method:ident($($params:tt)*)) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_) => $instance.$method($($params)*),
                ConfigState::InitialisedButUnAuthenticated(_) => return Err(CliError::UnAuthenticated),
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
    ($config:expr, $instance:ident.$method:ident($($params:tt)*).await) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_) => $instance.$method($($params)*).await,
                ConfigState::InitialisedButUnAuthenticated(_) => return Err(CliError::UnAuthenticated),
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
    ($config:expr, $function:ident($($params:tt)*)) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_) => $function($($params)*),
                ConfigState::InitialisedButUnAuthenticated(_) => return Err(CliError::UnAuthenticated),
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
    ($config:expr, $function:ident($($params:tt)*).await) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_) => $function($($params)*).await,
                ConfigState::InitialisedButUnAuthenticated(_) => return Err(CliError::UnAuthenticated),
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
}

macro_rules! must_login {
    ($config:expr, $instance:ident.$method:ident($($params:tt)*)) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_)
                | ConfigState::InitialisedButUnAuthenticated(_) => $instance.$method($($params)*),
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
    ($config:expr, $instance:ident.$method:ident($($params:tt)*).await) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_)
                | ConfigState::InitialisedButUnAuthenticated(_) => $instance.$method($($params)*).await,
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
    ($config:expr, $function:ident($($params:tt)*)) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_)
                | ConfigState::InitialisedButUnAuthenticated(_) => $function($($params)*),
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
    ($config:expr, $function:ident($($params:tt)*).await) => {
        {
            match $config {
                ConfigState::InitialisedAndAuthenticated(_)
                | ConfigState::InitialisedButUnAuthenticated(_) => $function($($params)*).await,
                ConfigState::Uninitialised => return Err(CliError::UnInitialised),
            }
        }
    };
}

#[derive(Default, Debug)]
pub struct GlobalContext {
    application: Option<String>,
    account: Option<String>,
    access_token: Option<String>,
}

#[tokio::main]
async fn main() {
    // Logger::new().init();

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
        CommandGroup::Pipeline(pipeline) => {
            must_login!(app.config, pipeline.handle_command(context).await)
        }
        CommandGroup::Config(config) => must_init!(app.config, config.handle_command(context)),
        CommandGroup::Init => {
            let existing_config = match app.config {
                ConfigState::InitialisedButUnAuthenticated(config)
                | ConfigState::InitialisedAndAuthenticated(config) => Some(config),
                ConfigState::Uninitialised => None,
            };
            handle_init(context, existing_config).await
        }
        CommandGroup::Login => must_init!(app.config, handle_login(context).await),
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

#![forbid(unsafe_code)]

mod app;
mod auth;
pub mod clap_app;
mod commands;
mod config;
mod error;
mod graphql;
mod loader;
mod output;
mod settings;
// mod logger;

use chrono::{DateTime, Local};
use commands::{
    completions::handle_completions, init::handle_init, login::handle_login, CommandGroup,
};
use config::{Config, CONFIG_FILE};
use error::CliError;
use output::error::display_error;
// use logger::Logger;
use crate::{auth::refresh_tokens, settings::Settings};
use app::{App, ConfigState};
use lazy_static::lazy_static;
use openidconnect::RefreshToken;
use std::process;

macro_rules! must_init {
    ($config:expr, $function_call:expr) => {{
        match $config {
            ConfigState::InitialisedAndAuthenticated(_)
            | ConfigState::InitialisedButUnAuthenticated(_) => $function_call,
            ConfigState::Uninitialised => return Err(CliError::UnInitialised),
        }
    }};
}

macro_rules! must_init_and_login {
    ($config:expr, $function_call:expr) => {{
        match $config {
            ConfigState::InitialisedAndAuthenticated(_) => $function_call,
            ConfigState::InitialisedButUnAuthenticated(_) => return Err(CliError::UnAuthenticated),
            ConfigState::Uninitialised => return Err(CliError::UnInitialised),
        }
    }};
}

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().unwrap();
}

#[derive(Default, Debug)]
pub struct GlobalContext {
    application: Option<String>,
    account: Option<String>,
    access_token: Option<String>,
    id_token: Option<String>,
}

#[tokio::main]
async fn main() {
    match run().await {
        Err(error) => {
            display_error(error);
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
    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");
    let app = App::new(config_file)?;

    let mut context = GlobalContext::default();
    let mut existing_config = None;

    match app.config {
        app::ConfigState::InitialisedAndAuthenticated(ref config) => {
            // check access token expiry
            let local: DateTime<Local> = Local::now();
            let expiry = DateTime::parse_from_rfc3339(&config.auth.as_ref().unwrap().expiry_time)
                .unwrap()
                .with_timezone(&Local);

            if local >= expiry {
                let new_tokens = refresh_tokens(&RefreshToken::new(
                    config.auth.as_ref().unwrap().refresh_token.clone(),
                ))
                .await
                .unwrap();

                match Config::load(config_file) {
                    Ok(mut config) => {
                        match config.auth.as_mut() {
                            Some(existing_auth_config) => {
                                existing_auth_config.refresh_token = new_tokens.refresh_token;
                                existing_auth_config.id_token = new_tokens.id_token;
                                existing_auth_config.access_token = new_tokens.access_token;
                                existing_auth_config.expiry_time = new_tokens.expiry_time;
                            }
                            None => {
                                // this shouldn't happen
                                panic!("Auth config is not avaliable.")
                            }
                        };
                        config.save(config_file).unwrap();
                        context.application = Some(config.core.application.clone());
                        context.account = Some(config.auth.as_ref().unwrap().account.clone());
                        context.id_token = Some(config.auth.as_ref().unwrap().id_token.clone());
                        context.access_token =
                            Some(config.auth.as_ref().unwrap().access_token.clone());
                        existing_config = Some(config);
                    }
                    Err(_) => {
                        // this shouldn't happen
                        panic!("Config is not avaliable.")
                    }
                };
            } else {
                context.application = Some(config.core.application.clone());
                context.account = Some(config.auth.as_ref().unwrap().account.clone());
                context.access_token = Some(config.auth.as_ref().unwrap().access_token.clone());
                context.id_token = Some(config.auth.as_ref().unwrap().id_token.clone());
                existing_config = Some(config.clone());
            }
        }
        app::ConfigState::InitialisedButUnAuthenticated(ref config) => {
            context.application = Some(config.core.application.clone());
            existing_config = Some(config.clone());
        }
        app::ConfigState::Uninitialised => {}
    };

    // overwritten by --application flag
    if let Some(ref application) = app.cli.application {
        context.application = Some(application.clone());
    }

    match app.cli.command_group {
        CommandGroup::Pipeline(pipeline) => {
            must_init_and_login!(app.config, pipeline.handle_command(context).await)
        }
        CommandGroup::Config(config) => must_init!(app.config, config.handle_command(context)),
        CommandGroup::Login => must_init!(app.config, handle_login(context).await),
        CommandGroup::Init => handle_init(context, existing_config).await,
        CommandGroup::Completions { shell } => handle_completions(context, shell),
        CommandGroup::Deployment(deployment) => {
            must_init_and_login!(app.config, deployment.handle_command(context).await)
        }
    }
}

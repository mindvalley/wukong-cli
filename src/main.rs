#![forbid(unsafe_code)]

mod app;
mod auth;
pub mod clap_app;
mod commands;
mod config;
mod error;
mod graphql;
mod loader;
mod logger;
mod output;
mod telemetry;

use crate::auth::refresh_tokens;
use app::{App, ConfigState};
use chrono::{DateTime, Local};
use clap::crate_version;
use commands::{
    completion::handle_completion, init::handle_init, login::handle_login, CommandGroup,
};
use config::{AuthConfig, Config, CONFIG_FILE};
use error::CliError;
use human_panic::setup_panic;
use log::{debug, error, info};
use openidconnect::RefreshToken;
use output::error::ErrorOutput;
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

#[derive(Default, Debug)]
pub struct GlobalContext {
    application: Option<String>,
    account: Option<String>,
    access_token: Option<String>,
    id_token: Option<String>,
    sub: Option<String>,
}

#[tokio::main]
async fn main() {
    setup_panic!();

    // make sure that the cursor re-appears when interrupting
    ctrlc::set_handler(move || {
        let term = dialoguer::console::Term::stdout();
        let _ = term.show_cursor();
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");

    match run().await {
        Err(error) => {
            error!("{}", ErrorOutput(error));
            process::exit(1);
        }
        Ok(false) => {
            info!("wukong cli session ended.");
            process::exit(1);
        }
        Ok(true) => {
            info!("wukong cli session ended.");
            process::exit(0);
        }
    }
}

async fn run() -> Result<bool, CliError> {
    let config_file = CONFIG_FILE
        .as_ref()
        .expect("Unable to identify user's home directory");
    let app = App::new(config_file)?;

    let mut context = GlobalContext::default();
    let mut existing_config = None;

    logger::Builder::new()
        .with_max_level(app.cli.verbose.log_level_filter())
        .init();

    match app.config {
        app::ConfigState::InitialisedAndAuthenticated(ref config) => {
            // SAFETY: the config state is authenticated so the auth must not be None here
            let auth_config = &config.auth.as_ref().unwrap();

            // check access token expiry
            let local: DateTime<Local> = Local::now();
            let expiry = DateTime::parse_from_rfc3339(&auth_config.expiry_time)
                .unwrap()
                .with_timezone(&Local);

            if local >= expiry {
                let new_tokens =
                    refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone())).await?;

                let mut updated_config = config.clone();
                updated_config.auth = Some(AuthConfig {
                    account: auth_config.account.clone(),
                    subject: auth_config.subject.clone(),
                    id_token: new_tokens.id_token.clone(),
                    access_token: new_tokens.access_token.clone(),
                    expiry_time: new_tokens.expiry_time,
                    refresh_token: new_tokens.refresh_token,
                });

                updated_config
                    .save(config_file)
                    .expect("The token is refreshed but the new config can't be saved.");
                context.application = Some(updated_config.core.application.clone());
                context.sub = Some(auth_config.subject.clone());
                context.account = Some(auth_config.account.clone());
                context.id_token = Some(new_tokens.id_token);
                context.access_token = Some(new_tokens.access_token);

                existing_config = Some(updated_config);
            } else {
                context.application = Some(config.core.application.clone());
                context.account = Some(auth_config.account.clone());
                context.sub = Some(auth_config.subject.clone());
                context.access_token = Some(auth_config.access_token.clone());
                context.id_token = Some(auth_config.id_token.clone());

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

    debug!("current cli version: {}", crate_version!());
    debug!("current application: {:?}", &context.application);
    debug!(
        "current API URL: {:?}",
        match &app.config {
            ConfigState::InitialisedButUnAuthenticated(config)
            | ConfigState::InitialisedAndAuthenticated(config) => Some(&config.core.wukong_api_url),
            ConfigState::Uninitialised => None,
        }
    );
    debug!("current calling user: {:?}", &context.account);

    match app.cli.command_group {
        CommandGroup::Pipeline(pipeline) => {
            must_init_and_login!(app.config, pipeline.handle_command(context).await)
        }
        CommandGroup::Config(config) => must_init!(app.config, config.handle_command(context)),
        CommandGroup::Login => must_init!(app.config, handle_login(context).await),
        CommandGroup::Init => handle_init(context, existing_config).await,
        CommandGroup::Completion { shell } => handle_completion(context, shell),
        CommandGroup::Deployment(deployment) => {
            must_init_and_login!(app.config, deployment.handle_command(context).await)
        }
    }
}

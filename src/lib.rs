mod app;
mod auth;
mod commands;
mod config;
mod error;
mod graphql;
mod loader;
mod logger;
pub mod output;
mod telemetry;

use crate::auth::refresh_tokens;
use app::{App, ConfigState};
use chrono::{DateTime, Local};
use clap::crate_version;
// use commands::{
//     completion::handle_completion, init::handle_init, login::handle_login, CommandGroup,
// };
use config::{AuthConfig, Config, CONFIG_FILE};
use error::CliError;
use log::debug;
use openidconnect::RefreshToken;

#[derive(Default, Debug)]
pub struct GlobalContext {
    application: Option<String>,
    overwrite_application: Option<String>,
    account: Option<String>,
    access_token: Option<String>,
    id_token: Option<String>,
    sub: Option<String>,
    config: Option<Config>,
}

impl GlobalContext {
    // fn config(&self) -> Option<Config> {
    //     todo!()
    // }

    // fn application(&self) -> Option<String> {
    //     match self.overwrite_application {
    //         Some(application) => Some(application),
    //         None => {
    //             todo!()
    //         }
    //     }
    // }

    // fn token(&self) -> Option<String> {
    //     self.id_token
    // }
}

pub async fn run() -> Result<bool, CliError> {
    // let config_file = CONFIG_FILE
    //     .as_ref()
    //     .expect("Unable to identify user's home directory");

    let app = App::new()?;

    let mut context = GlobalContext::default();
    // let mut existing_config = None;

    // match app.config {
    //     app::ConfigState::InitialisedAndAuthenticated(ref config) => {
    //         // SAFETY: the config state is authenticated so the auth must not be None here
    //         let auth_config = &config.auth.as_ref().unwrap();

    //         // check access token expiry
    //         let local: DateTime<Local> = Local::now();
    //         let expiry = DateTime::parse_from_rfc3339(&auth_config.expiry_time)
    //             .unwrap()
    //             .with_timezone(&Local);

    //         if local >= expiry {
    //             let new_tokens =
    //                 refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone())).await?;

    //             let mut updated_config = config.clone();
    //             updated_config.auth = Some(AuthConfig {
    //                 account: auth_config.account.clone(),
    //                 subject: auth_config.subject.clone(),
    //                 id_token: new_tokens.id_token.clone(),
    //                 access_token: new_tokens.access_token.clone(),
    //                 expiry_time: new_tokens.expiry_time,
    //                 refresh_token: new_tokens.refresh_token,
    //             });

    //             updated_config
    //                 .save(config_file)
    //                 .expect("The token is refreshed but the new config can't be saved.");
    //             context.application = Some(updated_config.core.application.clone());
    //             context.sub = Some(auth_config.subject.clone());
    //             context.account = Some(auth_config.account.clone());
    //             context.id_token = Some(new_tokens.id_token);
    //             context.access_token = Some(new_tokens.access_token);

    //             existing_config = Some(updated_config);
    //         } else {
    //             context.application = Some(config.core.application.clone());
    //             context.account = Some(auth_config.account.clone());
    //             context.sub = Some(auth_config.subject.clone());
    //             context.access_token = Some(auth_config.access_token.clone());
    //             context.id_token = Some(auth_config.id_token.clone());

    //             existing_config = Some(config.clone());
    //         }
    //     }
    //     app::ConfigState::InitialisedButUnAuthenticated(ref config) => {
    //         context.application = Some(config.core.application.clone());
    //         existing_config = Some(config.clone());
    //     }
    //     app::ConfigState::Uninitialised => {}
    // };

    // overwritten by --application flag
    if let Some(ref application) = app.cli.application {
        context.application = Some(application.clone());
    }

    debug!("current cli version: {}", crate_version!());
    debug!("current application: {:?}", &context.application);
    // debug!(
    //     "current API URL: {:?}",
    //     match &app.config {
    //         ConfigState::InitialisedButUnAuthenticated(config)
    //         | ConfigState::InitialisedAndAuthenticated(config) => Some(&config.core.wukong_api_url),
    //         ConfigState::Uninitialised => None,
    //     }
    // );
    debug!("current calling user: {:?}", &context.account);

    app.cli.run(context).await
}

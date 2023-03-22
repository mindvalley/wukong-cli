pub mod application;
pub mod completion;
pub mod config;
pub mod deployment;
pub mod dev;
pub mod init;
pub mod login;
pub mod pipeline;

use chrono::{DateTime, Local};
use clap::{command, crate_version, Parser, Subcommand};
use clap_complete::Shell;
use clap_verbosity_flag::{LogLevel, Verbosity};
use log::debug;
use openidconnect::RefreshToken;

use crate::{
    auth::Auth,
    config::{AuthConfig, Config, CONFIG_FILE},
    error::CliError,
};

use self::{completion::handle_completion, init::handle_init, login::handle_login};

#[derive(Default, Debug)]
pub struct State {
    application: Option<String>,
    sub: Option<String>,
}

#[derive(Default, Debug)]
pub struct Context {
    state: State,
    config: Config,
}

impl Context {
    pub async fn from_state(mut state: State) -> Result<Self, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let mut config = Config::load(config_file)?;

        let auth_config = config.auth.as_ref().ok_or(CliError::UnAuthenticated)?;

        // check access token expiry
        let local: DateTime<Local> = Local::now();
        let expiry = DateTime::parse_from_rfc3339(&auth_config.expiry_time)
            .unwrap()
            .with_timezone(&Local);

        if local >= expiry {
            let new_tokens = Auth::new(&config.core.okta_client_id)
                .refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone()))
                .await?;

            config.auth = Some(AuthConfig {
                account: auth_config.account.clone(),
                subject: auth_config.subject.clone(),
                id_token: new_tokens.id_token.clone(),
                access_token: new_tokens.access_token.clone(),
                expiry_time: new_tokens.expiry_time,
                refresh_token: new_tokens.refresh_token,
            });

            config
                .save(config_file)
                .expect("The token is refreshed but the new config can't be saved.");
        }

        if state.application.is_none() {
            state.application = Some(config.core.application.clone());
        }

        state.sub = Some(
            config
                .auth
                .as_ref()
                .ok_or(CliError::UnAuthenticated)?
                .subject
                .clone(),
        );

        debug!("current application: {:?}", &state.application);
        debug!("current API URL: {:?}", &config.core.wukong_api_url);
        debug!(
            "current calling user: {:?}",
            &config.auth.as_ref().unwrap().account
        );

        Ok(Context { state, config })
    }
}

/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Debug, Parser)]
#[command(version, author)]
pub struct ClapApp {
    #[command(subcommand)]
    pub command_group: CommandGroup,

    /// Override the application name that the CLI will perform the command against.
    /// If the flag is not used, then the CLI will use the default application name from the config.
    #[arg(long, short, global = true)]
    pub application: Option<String>,

    #[command(flatten)]
    pub verbose: Verbosity<ErrorLevel>,
}

#[derive(Debug)]
pub struct ErrorLevel;

impl LogLevel for ErrorLevel {
    fn default() -> Option<log::Level> {
        Some(log::Level::Error)
    }

    fn verbose_help() -> Option<&'static str> {
        Some("Use verbos output. More output per occurrence.\n\nBy default, it'll only report errors.\n`-v` show warnings\n`-vv` show info\n`-vvv` show debug\n`-vvvv` show trace")
    }

    fn verbose_long_help() -> Option<&'static str> {
        None
    }

    fn quiet_help() -> Option<&'static str> {
        Some("Do not print log message")
    }

    fn quiet_long_help() -> Option<&'static str> {
        None
    }
}

#[derive(Debug, Subcommand)]
pub enum CommandGroup {
    /// Initialize Wukong's configurations
    Init,
    /// This command group contains the commands to interact with an application’s configurations
    Application(application::Application),
    /// This command group contains the commands to view & interact with an application’s pipeline
    Pipeline(pipeline::Pipeline),
    /// This command group contains the commands to view and interact with the
    /// Continuous Delivery pipeline of an application.
    Deployment(deployment::Deployment),
    /// This command group contains the commands to interact with the local development environment.
    Dev(dev::Dev),
    /// This command group contains the commands to view & interact with Wukong's configurations
    Config(config::Config),
    /// Login to start using wukong command
    Login,
    /// Generate wukong cli completions for your shell to stdout
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
}

impl ClapApp {
    pub async fn execute(&self) -> Result<bool, CliError> {
        let mut state = State::default();

        // overwritten by --application flag
        if let Some(ref application) = self.application {
            state.application = Some(application.clone());
        }

        debug!("current cli version: {}", crate_version!());

        match &self.command_group {
            CommandGroup::Init => handle_init().await,
            CommandGroup::Completion { shell } => handle_completion(*shell),
            CommandGroup::Login => handle_login().await,
            CommandGroup::Application(application) => application.handle_command(state).await,
            CommandGroup::Pipeline(pipeline) => pipeline.handle_command(state).await,
            CommandGroup::Deployment(deployment) => deployment.handle_command(state).await,
            CommandGroup::Config(config) => config.handle_command(),
            CommandGroup::Dev(dev) => dev.handle_command(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ClapApp;

    #[test]
    fn verify_app() {
        use clap::CommandFactory;

        ClapApp::command().debug_assert()
    }
}

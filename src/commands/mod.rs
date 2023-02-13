pub mod application;
pub mod completion;
pub mod config;
pub mod deployment;
pub mod init;
pub mod login;
pub mod pipeline;

use clap::{command, crate_version, Parser, Subcommand};
use clap_complete::Shell;
use clap_verbosity_flag::{LogLevel, Verbosity};
use log::debug;

use crate::{config::Config, error::CliError};

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
        let state = State::default();

        // overwritten by --application flag
        // if let Some(ref application) = self.application {
        //     context.application = Some(application.clone());
        // }

        debug!("current cli version: {}", crate_version!());
        // debug!("current application: {:?}", &context.application);
        // debug!(
        //     "current API URL: {:?}",
        //     match &app.config {
        //         ConfigState::InitialisedButUnAuthenticated(config)
        //         | ConfigState::InitialisedAndAuthenticated(config) => Some(&config.core.wukong_api_url),
        //         ConfigState::Uninitialised => None,
        //     }
        // );
        // debug!("current calling user: {:?}", &context.account);

        match &self.command_group {
            CommandGroup::Init => handle_init().await,
            CommandGroup::Completion { shell } => handle_completion(*shell),
            CommandGroup::Login => handle_login().await,
            CommandGroup::Application(application) => application.handle_command(state).await,
            CommandGroup::Pipeline(pipeline) => pipeline.handle_command(state).await,
            CommandGroup::Deployment(deployment) => deployment.handle_command(state).await,
            CommandGroup::Config(config) => config.handle_command(),
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

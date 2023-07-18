use clap::{command, crate_version, Parser, Subcommand};
// use clap_complete::Shell;
use clap_verbosity_flag::{LogLevel, Verbosity};
use log::debug;

use crate::{
    commands::{init::handle_init, login::handle_login},
    config::Config,
    error::WKCliError,
};

mod application;
mod config;
mod deployment;
mod init;
mod login;
mod pipeline;

#[derive(Debug, Default)]
pub struct Context {
    current_application: String,
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
    // /// This command group contains the commands to view and interact with the
    /// Continuous Delivery pipeline of an application.
    Deployment(deployment::Deployment),
    // /// This command group contains the commands to interact with the local development environment.
    // Dev(dev::Dev),
    /// This command group contains the commands to view & interact with Wukong's configurations
    Config(config::Config),
    /// Login to start using wukong command
    Login,
    // /// Generate wukong cli completions for your shell to stdout
    // Completion {
    //     #[arg(value_enum)]
    //     shell: Shell,
    // },
}

impl ClapApp {
    pub async fn execute(&self) -> Result<bool, WKCliError> {
        let mut context = Context::default();

        // overwritten by --application flag
        context.current_application = match self.application {
            Some(ref application) => application.clone(),
            None => {
                let config = Config::load_from_default_path()?;
                config.core.application
            }
        };

        debug!("current cli version: {}", crate_version!());

        match &self.command_group {
            CommandGroup::Init => handle_init().await,
            // CommandGroup::Completion { shell } => handle_completion(*shell),
            CommandGroup::Login => handle_login().await,
            CommandGroup::Application(application) => application.handle_command(context).await,
            CommandGroup::Pipeline(pipeline) => pipeline.handle_command(context).await,
            CommandGroup::Deployment(deployment) => deployment.handle_command(context).await,
            CommandGroup::Config(config) => config.handle_command(),
            // CommandGroup::Dev(dev) => dev.handle_command().await,
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

pub mod application;
pub mod completion;
pub mod config;
pub mod deployment;
pub mod init;
pub mod login;
pub mod pipeline;

use clap::{command, Parser, Subcommand};
use clap_complete::Shell;
use clap_verbosity_flag::{LogLevel, Verbosity};

use crate::{
    app::APP_CONFIG,
    config::{Config, CONFIG_FILE},
    error::CliError,
};

use self::{completion::handle_completion, init::handle_init, login::handle_login};

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
    pub async fn run(&self, context: crate::GlobalContext) -> Result<bool, CliError> {
        match &self.command_group {
            CommandGroup::Init => handle_init().await,
            CommandGroup::Completion { shell } => handle_completion(context, *shell),
            CommandGroup::Login => handle_login().await,
            CommandGroup::Application(application) => todo!(),
            CommandGroup::Pipeline(pipeline) => pipeline.handle_command(context).await,
            CommandGroup::Deployment(_) => todo!(),
            CommandGroup::Config(_) => todo!(),
        }

        // match self.command_group {
        //     CommandGroup::Pipeline(pipeline) => {
        //         must_init_and_login!(app.config, pipeline.handle_command(context).await)
        //     }
        //     CommandGroup::Config(config) => must_init!(app.config, config.handle_command(context)),
        //     CommandGroup::Login => must_init!(app.config, handle_login(context).await),
        //     CommandGroup::Init => handle_init(context, existing_config).await,
        //     CommandGroup::Completion { shell } => handle_completion(context, shell),
        //     CommandGroup::Deployment(deployment) => {
        //         must_init_and_login!(app.config, deployment.handle_command(context).await)
        //     }
        //     CommandGroup::Application(application) => application.handle_command(context).await,
        // }
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

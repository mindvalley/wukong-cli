use clap::{command, crate_version, Parser, Subcommand};
use clap_complete::Shell;
use clap_verbosity_flag::{LogLevel, Verbosity};
use log::debug;

use crate::{
    application_config::ApplicationConfigs,
    commands::{
        completion::handle_completion, init::handle_init, login::handle_login, tui::handle_tui,
    },
    config::{ApiChannel, Config},
    error::WKCliError,
    update,
};

mod application;
mod completion;
mod config;
mod deployment;
mod dev;
mod google;
mod init;
mod login;
mod tui;

#[derive(Debug, Default)]
pub struct Context {
    pub current_application: String,
    pub sub: Option<String>,
    pub channel: ApiChannel,
}

/// A Swiss-army Knife CLI For Mindvalley Developers
#[derive(Debug, Parser)]
#[command(version, author)]
pub struct ClapApp {
    #[command(subcommand)]
    pub command_group: CommandGroup,

    #[command(flatten)]
    pub verbose: Verbosity<ErrorLevel>,

    /// Store the debugging log in the log file, which is located at ~/.config/wukong
    #[arg(long, global = true)]
    pub report: bool,

    /// Use the Canary channel API
    #[arg(long, global = true, default_value_t = true)]
    canary: bool,
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
    /// This command group contains the commands to view and interact with the
    /// Continuous Delivery pipeline of an application.
    Deployment(deployment::Deployment),
    /// This command group contains the commands to interact with the local development environment.
    Dev(dev::Dev),
    /// This command group contains the commands to view & interact with Wukong's configurations
    Config(config::Config),
    /// Login to start using wukong command
    Login,
    /// This command group contains the commands to interact with Google services
    Google(google::Google),
    /// Generate wukong cli completions for your shell to stdout
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Start TUI session
    Tui,
}

impl ClapApp {
    pub async fn execute(&self) -> Result<bool, WKCliError> {
        debug!("current cli version: {}", crate_version!());
        let channel = if self.canary {
            ApiChannel::Canary
        } else {
            ApiChannel::Stable
        };

        debug!("API channel: {:?}", channel);

        let command = match &self.command_group {
            CommandGroup::Init => handle_init().await,
            CommandGroup::Completion { shell } => handle_completion(*shell),
            CommandGroup::Login => handle_login().await,
            CommandGroup::Google(google) => google.handle_command().await,
            CommandGroup::Application(application) => application.handle_command(self).await,
            CommandGroup::Deployment(deployment) => {
                deployment.handle_command(get_context(self)?).await
            }
            CommandGroup::Config(config) => config.handle_command(),
            CommandGroup::Dev(dev) => dev.handle_command(self).await,
            CommandGroup::Tui => handle_tui(channel).await,
        };

        // Check for CLI updates:
        // Disabled check_for_update in test and dev as snapshots keep changing with each run
        if cfg!(feature = "prod") {
            update::check_for_update().await;
        }

        command
    }
}

// for telemetry
fn get_context(clap_app: &ClapApp) -> Result<Context, WKCliError> {
    let config = Config::load_from_default_path()?;

    let context = Context {
        current_application: {
            let application_configs = ApplicationConfigs::load()?;
            application_configs.application.name
        },
        sub: config.auth.okta.map(|auth_config| auth_config.subject),
        // if the `--canary` flag is used, then the CLI will use the Canary channel API,
        // otherwise, it will use the Stable channel API.
        channel: if clap_app.canary {
            ApiChannel::Canary
        } else {
            ApiChannel::Stable
        },
    };

    Ok(context)
}

// some command need to be executed without application context, e.g. `wukong application init` and `wukong dev config pull`
// otherwise, it will cause an error when trying to get the current application name
fn get_context_without_application(clap_app: &ClapApp) -> Result<Context, WKCliError> {
    let config = Config::load_from_default_path()?;

    let context = Context {
        current_application: "unknown".to_string(),
        sub: config.auth.okta.map(|auth_config| auth_config.subject),
        // if the `--canary` flag is used, then the CLI will use the Canary channel API,
        // otherwise, it will use the Stable channel API.
        channel: if clap_app.canary {
            ApiChannel::Canary
        } else {
            ApiChannel::Stable
        },
    };

    Ok(context)
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

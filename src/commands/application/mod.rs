pub mod info;
mod logs;
mod logs_demo;

use self::{logs::handle_logs, logs_demo::handle_logs_demo};
use super::{Context, State};
use crate::error::CliError;
use clap::{command, Args, Subcommand, ValueEnum};
use info::handle_info;

#[derive(Debug, Args)]
pub struct Application {
    #[command(subcommand)]
    pub subcommand: ApplicationSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ApplicationSubcommand {
    /// Show the application’s relevant informations
    Info,
    /// Demo gcloud log
    LogsDemo,
    /// Getting the logs of the applications from the Google Cloud Logging.
    Logs {
        /// (optional) The namespace to deploy to.
        #[arg(long, value_enum, default_value_t=ApplicationNamespace::Prod)]
        namespace: ApplicationNamespace,
        /// (optional) The version that the deployment will perform against.
        #[arg(long, value_enum, default_value_t=ApplicationVersion::Green)]
        version: ApplicationVersion,
        /// Only print out logs line with severity >= ERROR.
        #[arg(long)]
        errors: bool,
        /// Show logs lines newer from relative duration, e.g 5m, 1h, 1d.
        /// Also accept datetime in RFC 3339 format.
        #[arg(long, short)]
        since: Option<String>,
        /// Show logs lines older than relative duration, e.g 30m, 2h, 2d.
        /// Also accept datetime in RFC 3339 format.
        #[arg(long, short)]
        until: Option<String>,
        /// Limiting the number of log entries to return.  
        #[arg(long, default_value_t = 500)]
        limit: i32,
    },
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ApplicationVersion {
    Blue,
    Green,
}

impl ToString for ApplicationVersion {
    fn to_string(&self) -> String {
        match self {
            ApplicationVersion::Blue => "Blue".to_string(),
            ApplicationVersion::Green => "Green".to_string(),
        }
    }
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ApplicationNamespace {
    Prod,
    Staging,
}

impl ToString for ApplicationNamespace {
    fn to_string(&self) -> String {
        match self {
            ApplicationNamespace::Prod => "Prod".to_string(),
            ApplicationNamespace::Staging => "Staging".to_string(),
        }
    }
}

impl Application {
    pub async fn handle_command(&self, state: State) -> Result<bool, CliError> {
        let context = Context::from_state(state).await?;

        match &self.subcommand {
            ApplicationSubcommand::Info => handle_info(context).await,
            ApplicationSubcommand::LogsDemo => handle_logs_demo(context).await,
            ApplicationSubcommand::Logs {
                namespace,
                version,
                errors,
                since,
                until,
                limit,
            } => handle_logs(context, namespace, version, errors, since, until, limit).await,
        }
    }
}

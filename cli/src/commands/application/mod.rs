mod info;
mod init;
mod instances;
mod logs;

pub use self::logs::generate_filter;
use self::{init::handle_application_init, logs::handle_logs};
use clap::{command, Args, Subcommand, ValueEnum};

use crate::error::WKCliError;

use super::{get_context, get_context_without_application, ClapApp};
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
        /// (allow multiple flags) Logs lines to include.
        #[arg(long, short)]
        include: Vec<String>,
        /// (allow multiple flags) Logs lines to exclude.
        #[arg(long, short)]
        exclude: Vec<String>,
        /// Generate the URL to view the logs in browser.
        #[arg(long)]
        url_mode: bool,
    },
    /// This command group contains the commands to interact with an application’s instances
    Instances(instances::Instances),
    // This command init the application’s instances
    Init,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ApplicationVersion {
    Blue,
    Green,
}

impl ToString for ApplicationVersion {
    fn to_string(&self) -> String {
        match self {
            ApplicationVersion::Blue => "blue".to_string(),
            ApplicationVersion::Green => "green".to_string(),
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
            ApplicationNamespace::Prod => "prod".to_string(),
            ApplicationNamespace::Staging => "staging".to_string(),
        }
    }
}

impl Application {
    pub async fn handle_command(&self, clap_app: &ClapApp) -> Result<bool, WKCliError> {
        match &self.subcommand {
            ApplicationSubcommand::Info => handle_info(get_context(clap_app)?).await,
            ApplicationSubcommand::Logs {
                namespace,
                version,
                errors,
                since,
                until,
                limit,
                include,
                exclude,
                url_mode,
            } => {
                handle_logs(
                    get_context(clap_app)?,
                    namespace,
                    version,
                    errors,
                    since,
                    until,
                    limit,
                    include,
                    exclude,
                    url_mode,
                )
                .await
            }
            ApplicationSubcommand::Instances(instances) => {
                instances.handle_command(get_context(clap_app)?).await
            }
            ApplicationSubcommand::Init => {
                handle_application_init(get_context_without_application(clap_app)?).await
            }
        }
    }
}

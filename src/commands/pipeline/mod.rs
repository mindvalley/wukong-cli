pub mod ci_status;
pub mod describe;
pub mod list;

use self::{ci_status::handle_ci_status, describe::handle_describe, list::handle_list};
use crate::{
    auth::Auth,
    config::{AuthConfig, Config, CONFIG_FILE},
    error::CliError,
    output::table::{fmt_option_milliseconds, fmt_option_timestamp, fmt_timestamp},
};
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use clap::{Args, Subcommand};
use openidconnect::RefreshToken;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str};
use tabled::Tabled;

use super::{Context, State};

#[derive(Tabled, Serialize, Deserialize, Debug)]
struct PipelineData {
    name: String,
    #[tabled(display_with = "fmt_option_timestamp")]
    last_succeeded_at: Option<i64>,
    #[tabled(display_with = "fmt_option_timestamp")]
    last_failed_at: Option<i64>,
    #[tabled(display_with = "fmt_option_milliseconds")]
    last_duration: Option<i64>,
}

#[derive(Tabled)]
struct PipelineBranch {
    name: String,
    #[tabled(display_with = "fmt_option_timestamp")]
    last_succeed_at: Option<i64>,
    #[tabled(display_with = "fmt_option_timestamp")]
    last_failed_at: Option<i64>,
    #[tabled(display_with = "fmt_option_milliseconds")]
    last_duration: Option<i64>,
}

#[derive(Tabled)]
struct PipelinePullRequest {
    name: String,
    #[tabled(display_with = "fmt_option_timestamp")]
    last_succeed_at: Option<i64>,
    #[tabled(display_with = "fmt_option_timestamp")]
    last_failed_at: Option<i64>,
    #[tabled(display_with = "fmt_option_milliseconds")]
    last_duration: Option<i64>,
}

#[derive(Tabled)]
struct PipelineCiStatus {
    branch: String,
    pull_request: String,
    ci_status: String,
    build_url: String,
    #[tabled(display_with = "fmt_timestamp")]
    timestamp: i64,
}

struct JobBuild {
    build_number: i64,
    timestamp: i64,
    // wait_duration: Option<i64>,
    // build_duration: Option<i64>,
    // total_duration: Option<i64>,
    commit_id: Option<String>,
    commit_msg: Option<String>,
    // commit_author: Option<String>,
    result: String,
}

impl Display for JobBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let naive = NaiveDateTime::from_timestamp_opt(
            self.timestamp / 1000,
            (self.timestamp % 1000) as u32 * 1_000_000,
        )
        .unwrap();
        let started_at = DateTime::<Utc>::from_utc(naive, Utc);

        let commit_msg = match self.commit_msg {
            Some(ref msg) => msg,
            None => "No commit message",
        };
        let commit_id = match self.commit_id {
            Some(ref id) => &id[0..7],
            None => "",
        };

        write!(
            f,
            "[{}] #{} ({})\n{} (commit: {})\n",
            self.result,
            self.build_number,
            started_at.to_rfc2822(),
            commit_msg,
            commit_id,
        )
    }
}

#[derive(Debug, Args)]
pub struct Pipeline {
    #[command(subcommand)]
    pub subcommand: PipelineSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum PipelineSubcommand {
    /// List the current pipelines of the application
    List,
    /// Show the details of a pipeline
    Describe {
        /// The pipeline name
        name: String,
    },
    /// Show the build status and (possible) errors on branch ci pipeline
    CiStatus {
        /// Repository url
        #[arg(long)]
        repo_url: Option<String>,
        /// Branch name
        #[arg(long)]
        branch: Option<String>,
    },
}

impl Pipeline {
    pub async fn handle_command(&self, mut state: State) -> Result<bool, CliError> {
        let config_file = CONFIG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        let current_config = Config::load(config_file)?;

        let config = if let Some(auth_config) = current_config.auth.as_ref() {
            // check access token expiry
            let local: DateTime<Local> = Local::now();
            let expiry = DateTime::parse_from_rfc3339(&auth_config.expiry_time)
                .unwrap()
                .with_timezone(&Local);

            if local >= expiry {
                let new_tokens = Auth::new(&current_config.core.okta_client_id)
                    .refresh_tokens(&RefreshToken::new(auth_config.refresh_token.clone()))
                    .await?;

                let mut updated_config = current_config.clone();
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

                updated_config
            } else {
                current_config
            }
        } else {
            return Err(CliError::UnAuthenticated);
        };

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

        let context = Context { state, config };

        match &self.subcommand {
            PipelineSubcommand::List => handle_list(context).await,
            PipelineSubcommand::Describe { name } => handle_describe(context, name).await,
            PipelineSubcommand::CiStatus { repo_url, branch } => {
                handle_ci_status(context, repo_url, branch).await
            }
        }
    }
}

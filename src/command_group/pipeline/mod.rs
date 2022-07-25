pub mod ci_status;
pub mod describe;
pub mod list;

use crate::error::CliError;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str};
use tabled::Tabled;

use self::{ci_status::handle_ci_status, describe::handle_describe, list::handle_list};

// fn fmt_option<T: Display>(o: &Option<T>) -> String {
//     match o {
//         Some(s) => format!("{}", s),
//         None => "N/A".to_string(),
//     }
// }

fn fmt_option_milliseconds(o: &Option<i64>) -> String {
    match o {
        Some(s) => {
            let duration = Duration::milliseconds(*s);
            let seconds = duration.num_seconds() % 60;
            let minutes = (duration.num_seconds() / 60) % 60;
            format!("{} mins {} secs", minutes, seconds)
        }
        None => "N/A".to_string(),
    }
}

fn fmt_option_timestamp(o: &Option<i64>) -> String {
    match o {
        Some(s) => {
            let naive =
                NaiveDateTime::from_timestamp_opt(s / 1000, (s % 1000) as u32 * 1_000_000).unwrap();
            let dt = DateTime::<Utc>::from_utc(naive, Utc);
            format!("{}", dt.to_rfc3339())
        }
        None => "N/A".to_string(),
    }
}

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
    #[clap(subcommand)]
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
        #[clap(long)]
        repo_url: Option<String>,
        /// Branch name
        #[clap(long)]
        branch: Option<String>,
    },
}

impl Pipeline {
    pub async fn perform_action<'a>(&self) -> Result<bool, CliError<'a>> {
        match &self.subcommand {
            PipelineSubcommand::List => handle_list().await,
            PipelineSubcommand::Describe { name } => handle_describe(name).await,
            PipelineSubcommand::CiStatus { repo_url, branch } => {
                handle_ci_status(repo_url, branch).await
            }
        }
    }
}

use tabled::Tabled;
use wukong_telemetry_macro::wukong_telemetry;

use super::PipelineCiStatus;
use crate::{
    commands::Context,
    error::CliError,
    graphql::QueryClientBuilder,
    loader::new_spinner_progress_bar,
    output::{colored_println, table::TableOutput},
    telemetry::{self, TelemetryData, TelemetryEvent},
};
use std::process::Command as ProcessCommand;

#[wukong_telemetry(command_event = "pipeline_ci_status")]
pub async fn handle_ci_status(
    context: Context,
    repo_url: &Option<String>,
    branch: &Option<String>,
) -> Result<bool, CliError> {
    let repo_url = match repo_url {
        Some(url) => url.clone(),
        None => {
            let output = ProcessCommand::new("git")
                .arg("config")
                .args(["--get", "remote.origin.url"])
                .output()
                .expect("failed to execute `git config --get remote.origin.url` command");

            let mut repo_url = String::from_utf8(output.stdout).unwrap();
            repo_url.pop(); // remove trailing newline
            repo_url
        }
    };

    let branch = match branch {
        Some(branch) => branch.clone(),
        None => {
            let output = ProcessCommand::new("git")
                .args(["branch", "--show-current"])
                .output()
                .expect("failed to execute `git branch --show-current` command");

            let mut branch = String::from_utf8(output.stdout).unwrap();
            branch.pop(); // remove trailing newline
            branch
        }
    };

    println!("Current directory info");
    println!("repo url: {repo_url}");
    println!("branch: {branch}");
    println!();

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Fetching ci status ...");

    let client = QueryClientBuilder::default()
        .with_access_token(
            context
                .config
                .auth
                .ok_or(CliError::UnAuthenticated)?
                .id_token,
        )
        .with_sub(context.state.sub)
        .with_api_url(context.config.core.wukong_api_url)
        .build()?;

    let ci_status_resp = client
        .fetch_ci_status(&repo_url, &branch)
        .await?
        .data
        .unwrap()
        .ci_status;

    progress_bar.finish_and_clear();

    match ci_status_resp {
        Some(ci_status) => {
            let pipeline_ci_status = PipelineCiStatus {
                branch,
                pull_request: ci_status.name,
                ci_status: ci_status.result,
                build_url: ci_status.build_url,
                timestamp: ci_status.timestamp,
            };

            let table = TableOutput {
                title: Some("CI Status:".to_string()),
                header: None,
                data: vec![pipeline_ci_status],
            };

            colored_println!("{table}");
        }
        None => {
            #[derive(Tabled)]
            struct EmptyPipelineStatus<'a> {
                branch: &'a str,
                pull_request: &'a str,
                ci_status: &'a str,
                build_url: &'a str,
                timestamp: &'a str,
            }

            let pipeline_ci_status = EmptyPipelineStatus {
                branch: &branch,
                pull_request: "N/A",
                ci_status: "N/A",
                build_url: "N/A",
                timestamp: "N/A",
            };

            let table = TableOutput {
                title: Some("CI Status:".to_string()),
                header: None,
                data: vec![pipeline_ci_status],
            };

            colored_println!("{table}");
        }
    }

    Ok(true)
}

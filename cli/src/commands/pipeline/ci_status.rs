use wukong_sdk::{
    error::{APIError, WKError},
    graphql::ci_status_query,
};

use crate::{
    commands::{pipeline::PipelineCiStatus, Context},
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    output::{colored_println, table::TableOutput},
    wukong_client::WKClient,
};
use std::process::Command as ProcessCommand;
use wukong_telemetry::*;
use wukong_telemetry_macro::*;

#[wukong_telemetry(command_event = "pipeline_ci_status")]
pub async fn handle_ci_status(
    context: Context,
    repo_url: &Option<String>,
    branch: &Option<String>,
) -> Result<bool, WKCliError> {
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

    let fetch_loader = new_spinner();
    fetch_loader.set_message("Fetching ci status ...");

    let config = Config::load_from_default_path()?;
    let mut wk_client = WKClient::for_channel(&config, &context.channel)?;

    let ci_status_resp = match wk_client.fetch_ci_status(&repo_url, &branch).await {
        Ok(resp) => Ok(resp),
        Err(err) => match &err {
            WKCliError::WKSdkError(wk_sdk_error) => match wk_sdk_error {
                WKError::APIError(APIError::ApplicationNotFound) => Err(WKCliError::PipelineError(
                    crate::error::PipelineError::CIStatusApplicationNotFound,
                )),
                WKError::APIError(APIError::BuildNotFound) => {
                    Ok(ci_status_query::ResponseData { ci_status: None })
                }
                _ => Err(err),
            },
            _ => Err(err),
        },
    }?
    .ci_status;

    fetch_loader.finish_and_clear();

    let table = match ci_status_resp {
        Some(ci_status) => TableOutput {
            title: Some("CI Status:".to_string()),
            header: None,
            data: vec![PipelineCiStatus {
                branch,
                pull_request: Some(ci_status.name),
                ci_status: Some(ci_status.result),
                build_url: Some(ci_status.build_url),
                timestamp: Some(ci_status.timestamp),
            }],
        },
        None => TableOutput {
            title: Some("CI Status:".to_string()),
            header: None,
            data: vec![PipelineCiStatus {
                branch,
                pull_request: None,
                ci_status: None,
                build_url: None,
                timestamp: None,
            }],
        },
    };

    colored_println!("{table}");

    Ok(true)
}

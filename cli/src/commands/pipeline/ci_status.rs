use log::debug;
use wukong_sdk::WKClient;

use crate::{
    commands::{pipeline::PipelineCiStatus, Context},
    config::Config,
    error::WKCliError,
    loader::new_spinner,
    output::{colored_println, table::TableOutput},
    utils::wukong_sdk::FromWKCliConfig,
};
use std::process::Command as ProcessCommand;

// #[wukong_telemetry(command_event = "pipeline_ci_status")]
pub async fn handle_ci_status(
    _context: Context,
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
    let wk_client = WKClient::from_cli_config(&config);

    let ci_status_resp = wk_client
        .fetch_ci_status(&repo_url, &branch)
        .await
        .map_err(|err| match &err {
            wukong_sdk::error::WKError::APIError(api_error) => match api_error {
                wukong_sdk::error::APIError::ResponseError { code, message: _ } => {
                    if code == "application_config_not_defined" {
                        debug!("The application config is not defined. code: {code}");
                        WKCliError::ApplicationConfigNotDefined
                    } else {
                        err.into()
                    }
                }
                _ => err.into(),
            },
            _ => err.into(),
        })?
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

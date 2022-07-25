use super::PipelineCiStatus;
use crate::{error::CliError, graphql::pipeline::CiStatusQuery};
use std::process::Command;
use tabled::Table;

pub async fn handle_ci_status<'a>(
    repo_url: &Option<String>,
    branch: &Option<String>,
) -> Result<bool, CliError<'a>> {
    let repo_url = match repo_url {
        Some(url) => url.clone(),
        None => {
            let output = Command::new("git")
                .arg("config")
                .args(["--get", "remote.origin.url"])
                .output()
                .expect("failed to execute `git config --get remote.origin.url` command");

            String::from_utf8(output.stdout).unwrap()
        }
    };

    let branch = match branch {
        Some(branch) => branch.clone(),
        None => {
            let output = Command::new("git")
                .args(["branch", "--show-current"])
                .output()
                .expect("failed to execute `git branch --show-current` command");

            String::from_utf8(output.stdout).unwrap()
        }
    };

    let ci_status_resp = CiStatusQuery::fetch(&repo_url, &branch)
        .await?
        .data
        // .ok_or(anyhow::anyhow!("Error"))?
        .unwrap()
        .ci_status;

    if let Some(ci_status) = ci_status_resp {
        let pipeline_ci_status = PipelineCiStatus {
            branch,
            pull_request: ci_status.name,
            ci_status: ci_status.result,
            build_url: ci_status.build_url,
            timestamp: ci_status.timestamp,
        };

        let table = Table::new([pipeline_ci_status]).to_string();

        println!("CI Status: ");
        println!("{table}");
    } else {
        println!("No result.")
    }

    Ok(true)
}

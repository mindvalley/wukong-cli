use super::PipelineCiStatus;
use crate::{error::CliError, graphql::QueryClientBuilder, GlobalContext};
use std::process::Command;
use tabled::{style::Style, Table};

pub async fn handle_ci_status<'a>(
    context: GlobalContext,
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

            let mut repo_url = String::from_utf8(output.stdout).unwrap();
            repo_url.pop(); // remove trailing newline
            repo_url
        }
    };

    let branch = match branch {
        Some(branch) => branch.clone(),
        None => {
            let output = Command::new("git")
                .args(["branch", "--show-current"])
                .output()
                .expect("failed to execute `git branch --show-current` command");

            let mut branch = String::from_utf8(output.stdout).unwrap();
            branch.pop(); // remove trailing newline
            branch
        }
    };

    let client = QueryClientBuilder::new()
        .with_access_token(context.id_token.unwrap())
        .build()?;

    let ci_status_resp = client
        .fetch_ci_status(&repo_url, &branch)
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

        let table = Table::new([pipeline_ci_status])
            .with(Style::modern())
            .to_string();

        println!("CI Status: ");
        println!("{table}");
    } else {
        println!("No result.")
    }

    Ok(true)
}

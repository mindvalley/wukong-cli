use crate::{error::CliError, graphql::pipeline::CiStatusQuery};
use std::process::Command;

pub async fn handle_ci_status<'a>(
    repo_url: &Option<String>,
    branch: &Option<String>,
) -> Result<bool, CliError<'a>> {
    println!("CI Status");

    let repo_url = match repo_url {
        Some(url) => url.clone(),
        None => {
            let output = Command::new("git")
                .arg("config")
                .args(["--get", "remote.origin.url"])
                .output()
                .expect("failed to execute process");

            String::from_utf8(output.stdout).unwrap()
        }
    };

    let branch = match branch {
        Some(branch) => branch.clone(),
        None => {
            let output = Command::new("git")
                .args(["branch", "--show-current"])
                .output()
                .expect("failed to execute process");

            String::from_utf8(output.stdout).unwrap()
        }
    };

    println!("repo_url: {:?}", repo_url);
    println!("branch: {:?}", branch);

    let ci_status_resp = CiStatusQuery::fetch(repo_url, branch)
        .await?
        .data
        // .ok_or(anyhow::anyhow!("Error"))?
        .unwrap()
        .ci_status;

    println!("{:?}", ci_status_resp);

    if let Some(ci_status) = ci_status_resp {
    } else {
        println!("No result.")
    }

    Ok(true)
}

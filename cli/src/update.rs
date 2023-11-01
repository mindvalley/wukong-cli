use crate::{
    config::{Config, ReleaseInfo},
    error::WKCliError,
    output::colored_print,
    utils::compare_with_current_time,
};
use aion::*;
use chrono::Utc;
use clap::crate_version;
use log::debug;
use owo_colors::OwoColorize;
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};

const WUKONG_GITHUB_REPO: &str = "mindvalley/wukong-cli";
const GITHUB_API_URL: &str = "https://api.github.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubLatestReleaseInfo {
    pub tag_name: String,
    pub url: String,
    pub published_at: String,
    pub html_url: String,
}

// check_for_update checks whether this wukong has had a newer release on Github
pub async fn check_for_update() -> Result<(), WKCliError> {
    let release_info = get_current_release_info()?;

    if let Some(release_info) = release_info {
        let last_update_checked_since =
            compare_with_current_time(&release_info.checked_for_update_at);

        if last_update_checked_since >= -24.hours() {
            debug!("No need to check for update");
            return Ok(());
        }
    }

    debug!("Checking for update");

    if let Some(latest_release_info) = get_latest_release_info(Some(GITHUB_API_URL)).await? {
        let current_version = crate_version!().to_string();
        let has_update = version_greater_than(&latest_release_info.version, &current_version);

        if has_update {
            debug!("New release found");
            colored_print!(
                "{} {} {} {}\n",
                "A new release of wukong is available:".yellow(),
                latest_release_info.version.cyan(),
                "→".cyan(),
                current_version.cyan()
            );

            colored_print!("To upgrade, run: brew upgrade wukong\n");
            colored_print!("{}\n", latest_release_info.url.yellow());
        } else {
            debug!("No new release found");
        }

        update_release_info(latest_release_info)?;
    }

    Ok(())
}

fn update_release_info(release_info: ReleaseInfo) -> Result<(), WKCliError> {
    let mut config = Config::load_from_default_path()?;
    config.release_info = Some(release_info);
    config.save_to_default_path()?;

    Ok(())
}

fn version_greater_than(new_version: &str, current_version: &str) -> bool {
    if let (Ok(new_version), Ok(current_version)) =
        (Version::parse(new_version), Version::parse(current_version))
    {
        new_version > current_version
    } else {
        false
    }
}

fn get_current_release_info() -> Result<Option<ReleaseInfo>, WKCliError> {
    let config = Config::load_from_default_path()?;
    Ok(config.release_info)
}

async fn get_latest_release_info(
    github_api_url: Option<&str>,
) -> Result<Option<ReleaseInfo>, WKCliError> {
    let client = Client::new();

    let url = format!(
        "{}/repos/{}/releases/latest",
        github_api_url.unwrap_or(GITHUB_API_URL),
        WUKONG_GITHUB_REPO
    );

    let response = client
        .get(&url)
        .header("user-agent", "wukong-cli")
        .send()
        .await?;

    if response.status().is_success() {
        let github_release_info = response
            .json::<GithubLatestReleaseInfo>()
            .await
            .map_err(|e| {
                debug!("Error: {:?}", e);
                println!("Error: {:?}", e);
            });

        if let Ok(github_release_info) = github_release_info {
            return Ok(Some(ReleaseInfo {
                version: github_release_info.tag_name,
                url: github_release_info.html_url,
                published_at: github_release_info.published_at,
                checked_for_update_at: Utc::now().to_rfc3339(),
            }));
        }
    } else {
        let message = response.text().await?;
        debug!("Error: {:?}", message);
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[test]
    fn test_version_greater_than() {
        // Test cases for version comparison.
        assert!(version_greater_than("1.1.0", "1.0.0"));
        assert!(!version_greater_than("1.0.0", "1.1.0"));
        assert!(!version_greater_than("1.0.0", "1.0.0"));
        assert!(!version_greater_than("invalid", "1.0.0"));
        assert!(!version_greater_than("1.0.0", "invalid"));
    }

    #[tokio::test]
    async fn test_get_latest_release_info() {
        let server = MockServer::start();

        let api_resp = r#"{
            "url": "https://github.com/mindvalley/wukong-cli/releases/tag/1.2.0",
            "html_url": "https://github.com/mindvalley/wukong-cli/releases/tag/1.2.0",
            "id": 120063991,
            "tag_name": "1.2.0",
            "published_at": "2023-09-06T07:08:46Z",
            "body": null
        }"#;

        let url = format!("/repos/{}/releases/latest", WUKONG_GITHUB_REPO);

        let mock_server = server.mock(|when, then| {
            when.method(GET)
                .path(url)
                .header("user-agent", "wukong-cli");
            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(api_resp);
        });
        println!("{:?}", &server.base_url());

        let release_info = get_latest_release_info(Some(&server.base_url())).await;

        mock_server.assert();
        assert!(release_info.is_ok());

        let release_info = release_info.unwrap();
        println!("release_info: {:?}", release_info);
        assert!(release_info.is_some());

        let release_info = release_info.unwrap();
        assert_eq!(release_info.version, "1.2.0");
        assert_eq!(release_info.published_at, "2023-09-06T07:08:46Z");
        assert_eq!(
            release_info.url,
            "https://github.com/mindvalley/wukong-cli/releases/tag/1.2.0"
        );
    }
}

use crate::{
    config::{Config, ReleaseInfo},
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
pub async fn check_for_update() {
    let release_info = get_current_release_info();

    if let Some(release_info) = release_info {
        let last_update_checked_since =
            compare_with_current_time(&release_info.last_update_checked_at);

        if last_update_checked_since >= -(72.hours()) {
            debug!("No need to check for update");
            return;
        }
    }

    debug!("Checking for update");

    if let Some(latest_release_info) = get_latest_release_info(Some(GITHUB_API_URL)).await {
        update_release_info(latest_release_info);
    }

    print_update_message();
}

fn print_update_message() {
    let release_info = get_current_release_info();
    let current_version = crate_version!().to_string();

    if let Some(release_info) = release_info {
        let has_update = version_greater_than(&release_info.version, &current_version);

        if has_update {
            debug!("New release found");
            colored_print!(
                "{} {} {} {}\n",
                "A new release of wukong is available:".yellow(),
                current_version.cyan(),
                "→".cyan(),
                release_info.version.cyan(),
            );

            colored_print!("To upgrade, run: brew upgrade wukong\n");
            colored_print!("{}\n", release_info.url.yellow());
        } else {
            debug!("No new release found");
        }
    }
}

fn get_current_release_info() -> Option<ReleaseInfo> {
    let config = Config::load_from_default_path().map_err(|e| {
        debug!("Error: {:?}", e);
    });

    if let Ok(config) = config {
        return config.release_info;
    }

    None
}

fn update_release_info(release_info: ReleaseInfo) {
    let config = Config::load_from_default_path();

    match config {
        Ok(mut config) => {
            config.release_info = Some(release_info);

            let _ = config.save_to_default_path().map_err(|e| {
                debug!("Error: {:?}", e);
            });
        }
        Err(e) => {
            debug!("Error: {:?}", e);
        }
    };
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

async fn get_latest_release_info(github_api_url: Option<&str>) -> Option<ReleaseInfo> {
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
        .await
        .map_err(|e| {
            debug!("Error: {:?}", e);
        });

    if let Ok(response) = response {
        let github_release_info = response
            .json::<GithubLatestReleaseInfo>()
            .await
            .map_err(|e| {
                debug!("Error: {:?}", e);
            });

        if let Ok(github_release_info) = github_release_info {
            return Some(ReleaseInfo {
                version: github_release_info.tag_name,
                url: github_release_info.html_url,
                published_at: github_release_info.published_at,
                last_update_checked_at: Utc::now().to_rfc3339(),
            });
        }
    }

    None
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

        let release_info = release_info.unwrap();
        assert_eq!(release_info.version, "1.2.0");
        assert_eq!(release_info.published_at, "2023-09-06T07:08:46Z");
        assert_eq!(
            release_info.url,
            "https://github.com/mindvalley/wukong-cli/releases/tag/1.2.0"
        );
    }
}

mod common;

use std::env;

use aion::*;
use assert_fs::prelude::*;
use httpmock::prelude::*;

#[test]
fn test_wukong_application_help() {
    let cmd = common::wukong_raw_command()
        .arg("application")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_application_info_success() {
    let server = MockServer::start();

    let api_resp = r#"
{
  "data": {
    "application": {
      "basicInfo": {
        "deploymentStrategy": "basic",
        "deploymentTarget": "kubernetes",
        "links": [
          {
            "title": "Performance Dashboard",
            "url": "https://grafana.mv.tech/aaa"
          },
          {
            "title": "SLOs Dashboard",
            "url": "https://grafana.mv.tech/bbb"
          },
          {
            "title": "Honeycomb Telemetry",
            "url": "https://ui.honeycomb.io/mv/datasets/ccc"
          }
        ]
      },
      "name": "valid-application"
    }
  }
}"#;

    let mock = server.mock(|when, then| {
        when.method(POST).path("/");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(api_resp);
    });

    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            format!(
                r#"
[core]
wukong_api_url = "{}"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "{}"
refresh_token = "refresh_token"
"#,
                server.base_url(),
                2.days().from_now().to_rfc3339()
            )
            .as_str(),
        )
        .unwrap();

    let application_config_file = temp.child(".wukong.toml");
    application_config_file.touch().unwrap();

    application_config_file
        .write_str(
            r#"
[application]
name = "valid-application"
enable = true

[[application.namespaces]]
type = "prod"
"#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("application")
        .arg("info")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .env("WUKONG_DEV_APP_CONFIG_FILE", application_config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    mock.assert();

    temp.close().unwrap();
}

#[test]
fn test_wukong_application_info_should_failed_when_application_config_not_exist() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            format!(
                r#"
[core]
wukong_api_url = "https://wukong-api.com"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "{}"
refresh_token = "refresh_token"
    "#,
                2.days().from_now().to_rfc3339()
            )
            .as_str(),
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("application")
        .arg("info")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_application_info_should_failed_when_unauthenticated() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
wukong_api_url = "https://wukong-api.com"

[auth]
"#,
        )
        .unwrap();

    let application_config_file = temp.child(".wukong.toml");
    application_config_file.touch().unwrap();

    application_config_file
        .write_str(
            r#"
[application]
name = "valid-application"
enable = true

[[application.namespaces]]
type = "prod"
"#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("application")
        .arg("info")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .env("WUKONG_DEV_APP_CONFIG_FILE", application_config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

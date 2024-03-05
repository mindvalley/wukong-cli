mod common;

use aion::*;
use assert_fs::prelude::*;
use httpmock::prelude::*;

#[test]
fn test_wukong_deployment_help() {
    let cmd = common::wukong_raw_command()
        .arg("deployment")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_deployment_list_success() {
    let server = MockServer::start();

    let api_resp = r#"
    {
      "data": {
        "cdPipelines": [
          {
            "deployedRef": null,
            "enabled": true,
            "environment": "prod",
            "lastDeployment": 1663161661001,
            "name": "pipeline-blue",
            "status": "TERMINAL",
            "version": "blue",
            "buildArtifact": "master-build-250"
          },
          {
            "deployedRef": null,
            "enabled": true,
            "environment": "prod",
            "lastDeployment": null,
            "name": "pipeline-green",
            "status": null,
            "version": "green",
            "buildArtifact": "master-build-1235"
          }
        ]
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
        .arg("deployment")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .env("WUKONG_DEV_APP_CONFIG_FILE", application_config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::with_settings!({filters => vec![
        (r"\d+ months ago|a year ago|\d+ years ago", "[DEPLOYMENT_TIME]"),
    ]}, {
        insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
    });

    mock.assert();
    temp.close().unwrap();
}

#[test]
fn test_wukong_deployment_list_should_failed_when_unauthenticated() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
application = "valid-application"
wukong_api_url = "https://wukong-api.com"

[auth]
    "#
            .to_string()
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
        .arg("deployment")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .env("WUKONG_DEV_APP_CONFIG_FILE", application_config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_deployment_list_should_failed_when_config_file_not_exist() {
    let cmd = common::wukong_raw_command()
        .arg("deployment")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", "/path/to/non/exist/config.toml")
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
}

#[test]
fn test_wukong_deployment_list_should_failed_when_application_config_file_not_exist() {
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
        .arg("deployment")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
}

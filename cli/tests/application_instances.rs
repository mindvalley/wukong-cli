mod common;

use aion::*;
use assert_fs::prelude::*;
use httpmock::prelude::*;

#[test]
fn test_wukong_application_instances_help() {
    let cmd = common::wukong_raw_command()
        .arg("application")
        .arg("instances")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_application_instances_list_success() {
    let server = MockServer::start();

    let kubernetes_pods_api_resp = r#"
{
  "data": {
    "kubernetesPods": [
      {
        "hostIp": "10.0.128.11",
        "name": "the-blue-1",
        "ready": true,
        "labels": ["label1", "label2"]
      },
      {
        "hostIp": null,
        "name": "the-blue-2",
        "ready": false,
        "labels": ["label1", "label2"]
      }
    ]
  }
}"#;

    let is_authorized_api_resp = r#"
{
  "data": {
    "isAuthorized": true
  }
}"#;

    let is_authorized_mock = server.mock(|when, then| {
        when.method(POST).path("/").body_contains("isAuthorized");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(is_authorized_api_resp);
    });
    let kubernetes_pods_mock = server.mock(|when, then| {
        when.method(POST).path("/").body_contains("kubernetesPods");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(kubernetes_pods_api_resp);
    });

    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            format!(
                r#"
[core]
application = "valid-application"
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

    let cmd = common::wukong_raw_command()
        .arg("application")
        .arg("instances")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    is_authorized_mock.assert();
    kubernetes_pods_mock.assert();

    temp.close().unwrap();
}

#[test]
fn test_wukong_application_instances_list_failed_if_not_authorized() {
    let server = MockServer::start();

    let is_authorized_api_resp = r#"
{
  "data": {
    "isAuthorized": false
  }
}"#;

    let is_authorized_mock = server.mock(|when, then| {
        when.method(POST).path("/").body_contains("isAuthorized");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(is_authorized_api_resp);
    });

    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            format!(
                r#"
[core]
application = "valid-application"
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

    let cmd = common::wukong_raw_command()
        .arg("application")
        .arg("instances")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    is_authorized_mock.assert();

    temp.close().unwrap();
}

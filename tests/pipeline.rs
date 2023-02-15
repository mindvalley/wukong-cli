mod common;

use aion::*;
use assert_fs::prelude::*;
use httpmock::prelude::*;

#[test]
fn test_wukong_pipeline_list() {
    let server = MockServer::start();

    let api_resp = r#"
    {
      "data": {
        "pipelines": [
          {
            "__typename": "MultiBranchPipeline",
            "lastDuration": null,
            "lastFailedAt": null,
            "lastSucceededAt": null,
            "name": "mv-platform-ci"
          },
          {
            "__typename": "Job",
            "lastDuration": 522303,
            "lastFailedAt": 1663844109893,
            "lastSucceededAt": 1664266988871,
            "name": "mv-platform-prod-main-branch-build"
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
    application = "valid-application"
    wukong_api_url = "{}"
    okta_client_id = "valid-okta-client-id"

    [auth]
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

    common::wukong_raw_command()
        .arg("pipeline")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    mock.assert();
    temp.close().unwrap();
}

#[test]
fn test_wukong_pipeline_describe() {
    let server = MockServer::start();

    let api_resp = r#"
    {
      "data": {
        "pipeline": {
            "__typename": "Job",
            "lastDuration": 522303,
            "lastFailedAt": 1663844109893,
            "lastSucceededAt": null,
            "name": "mv-platform-main-branch-build"
        }
      }
    }
    "#;

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
    application = "valid-application"
    wukong_api_url = "{}"
    okta_client_id = "valid-okta-client-id"

    [auth]
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

    common::wukong_raw_command()
        .arg("pipeline")
        .arg("describe")
        .arg("xxx")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    mock.assert();
    temp.close().unwrap();
}

#[test]
fn test_wukong_pipeline_ci_status() {
    let server = MockServer::start();

    let api_resp = r#"
    {
      "data": {
        "ciStatus": {
          "buildDuration": 582271,
          "buildNumber": 101,
          "buildUrl": "https://ci.mv.dev/mv-platform-ci/job/main/101/",
          "commits": [],
          "name": "main",
          "result": "SUCCESS",
          "timestamp": 1664267841689,
          "totalDuration": 582274,
          "waitDuration": 0
        }
      }
    }
    "#;

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
    application = "valid-application"
    wukong_api_url = "{}"
    okta_client_id = "valid-okta-client-id"

    [auth]
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

    common::wukong_raw_command()
        .arg("pipeline")
        .arg("ci-status")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    mock.assert();
    temp.close().unwrap();
}

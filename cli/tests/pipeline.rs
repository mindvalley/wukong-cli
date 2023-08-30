mod common;

use aion::*;
use assert_fs::prelude::*;
use httpmock::prelude::*;

#[test]
fn test_wukong_pipeline_help() {
    let cmd = common::wukong_raw_command()
        .arg("pipeline")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_pipeline_list_success() {
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

    let cmd = common::wukong_raw_command()
        .arg("pipeline")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .env("WUKONG_DEV_TIMEZONE", "Asia/Kuala_Lumpur")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    mock.assert();
    temp.close().unwrap();
}

#[test]
fn test_wukong_pipeline_list_should_failed_when_unauthenticated() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
application = "valid-application"
wukong_api_url = "https://wukong-api.com"
okta_client_id = "valid-okta-client-id"
"#
            .to_string()
            .as_str(),
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("pipeline")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_pipeline_describe_should_failed_when_unauthenticated() {
    let server = MockServer::start();

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
"#,
                server.base_url(),
            )
            .as_str(),
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("pipeline")
        .arg("describe")
        .arg("pipeline-xxx")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_pipeline_describe_success() {
    let server = MockServer::start();

    let api_resp = r#"
    {
      "data": {
        "pipeline": {
          "__typename": "Job",
          "builds": [
            {
              "buildDuration": 974066,
              "buildNumber": 6,
              "commits": [
                {
                  "author": "noreply@github.com",
                  "id": "2249f1a567b7bb606617a1e8875ab624ddb2f011",
                  "messageHeadline": "commit d"
                }
              ],
              "result": "SUCCESS",
              "timestamp": 1676525712091,
              "totalDuration": 982749,
              "waitDuration": 8680
            },
            {
              "buildDuration": 939446,
              "buildNumber": 5,
              "commits": [
                {
                  "author": "noreply@github.com",
                  "id": "f5bbe4a49e4ab1e47b51867d89ac971222212b88",
                  "messageHeadline": "commit c"
                }
              ],
              "result": "SUCCESS",
              "timestamp": 1676452389704,
              "totalDuration": 945240,
              "waitDuration": 5791
            },
            {
              "buildDuration": 932855,
              "buildNumber": 4,
              "commits": [
                {
                  "author": "noreply@github.com",
                  "id": "1388f051f6f40a9e82e208edb374cf7dd22e9c27",
                  "messageHeadline": "commit b"
                }
              ],
              "result": "SUCCESS",
              "timestamp": 1676445021410,
              "totalDuration": 1130569,
              "waitDuration": 5656
            },
            {
              "buildDuration": 1032073,
              "buildNumber": 3,
              "commits": [
                {
                  "author": "noreply@github.com",
                  "id": "75203226a925bc1f8d61d295dbf337a7b16986f6",
                  "messageHeadline": "commit a"
                }
              ],
              "result": "SUCCESS",
              "timestamp": 1676443989314,
              "totalDuration": 1037210,
              "waitDuration": 5134
            }
          ],
          "lastDuration": 974066,
          "lastFailedAt": 1676376311798,
          "lastSucceededAt": 1676525712091,
          "name": "pipeline-xxx"
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

    let cmd = common::wukong_raw_command()
        .arg("pipeline")
        .arg("describe")
        .arg("pipeline-xxx")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    mock.assert();
    temp.close().unwrap();
}

#[test]
fn test_wukong_pipeline_ci_status_success() {
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

    let root = assert_fs::TempDir::new().unwrap();
    let repo = root.child("new_repo");
    repo.create_dir_all().unwrap();

    git2::Repository::init(repo.path().to_str().unwrap()).unwrap();

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

    let cmd = common::wukong_raw_command()
        .arg("pipeline")
        .arg("ci-status")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .env("WUKONG_DEV_TIMEZONE", "Asia/Kuala_Lumpur")
        .current_dir(&repo)
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    mock.assert();
    root.close().unwrap();
    temp.close().unwrap();
}

#[test]
fn test_wukong_pipeline_ci_status_should_failed_when_unauthenticated() {
    let server = MockServer::start();

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
"#,
                server.base_url(),
            )
            .as_str(),
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("pipeline")
        .arg("ci-status")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

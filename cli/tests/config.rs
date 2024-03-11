mod common;

use assert_fs::prelude::*;

#[test]
fn test_wukong_config_help() {
    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_config_list_success() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
wukong_api_url = "https://wukong-api.com"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "2023-02-19T06:55:51.501915+00:00"
refresh_token = "refresh_token"
    "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_config_list_success_without_login() {
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

    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_config_list_should_failed_without_config_file() {
    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("list")
        .env("WUKONG_DEV_CONFIG_FILE", "/path/to/non/exist/config.toml")
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
}

#[test]
fn test_wukong_config_get_success() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
wukong_api_url = "https://wukong-api.com"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "2023-02-19T06:55:51.501915+00:00"
refresh_token = "refresh_token"
    "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("get")
        .arg("wukong-api-url")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_config_get_should_failed_with_non_supported_field() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
wukong_api_url = "https://wukong-api.com"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "2023-02-19T06:55:51.501915+00:00"
refresh_token = "refresh_token"
    "#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("get")
        .arg("access_token")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

#[test]
fn test_wukong_config_set_success_with_supported_field() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
wukong_api_url = "https://wukong-api.com"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "2023-02-19T06:55:51.501915+00:00"
refresh_token = "refresh_token"
"#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("set")
        .arg("wukong-api-url")
        .arg("https://new-wukong-api.com")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());

    config_file.assert(
        r#"[core]
wukong_api_url = "https://new-wukong-api.com"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "2023-02-19T06:55:51.501915+00:00"
refresh_token = "refresh_token"
"#,
    );

    temp.close().unwrap();
}

#[test]
fn test_wukong_config_set_should_failed_with_non_supported_field() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("config.toml");
    config_file.touch().unwrap();

    config_file
        .write_str(
            r#"
[core]
wukong_api_url = "https://wukong-api.com"

[auth.okta]
client_id = "valid-okta-client-id"
account = "test@email.com"
subject = "subject"
id_token = "id_token"
access_token = "access_token"
expiry_time = "2023-02-19T06:55:51.501915+00:00"
refresh_token = "refresh_token"
"#,
        )
        .unwrap();

    let cmd = common::wukong_raw_command()
        .arg("config")
        .arg("set")
        .arg("access_token")
        .arg("new-token")
        .env("WUKONG_DEV_CONFIG_FILE", config_file.path())
        .assert()
        .failure();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stderr).unwrap());

    temp.close().unwrap();
}

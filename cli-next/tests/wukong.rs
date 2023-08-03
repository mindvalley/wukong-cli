mod common;

#[test]
fn test_wukong_help_without_config_file() {
    let cmd = common::wukong_raw_command()
        .arg("help")
        .env("WUKONG_DEV_CONFIG_FILE", "path/to/non/exists/config.toml")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_version_without_config_file() {
    let cmd = common::wukong_raw_command()
        .arg("--version")
        .env("WUKONG_DEV_CONFIG_FILE", "path/to/non/exists/config.toml")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

mod common;

#[test]
fn test_wukong_dev_help() {
    let cmd = common::wukong_raw_command()
        .arg("dev")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

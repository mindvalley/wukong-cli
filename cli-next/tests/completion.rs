mod common;

#[test]
fn test_wukong_completion_help() {
    let cmd = common::wukong_raw_command()
        .arg("completion")
        .arg("--help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_completion_bash() {
    let cmd = common::wukong_raw_command()
        .arg("completion")
        .arg("bash")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_completion_zsh() {
    let cmd = common::wukong_raw_command()
        .arg("completion")
        .arg("zsh")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_completion_fish() {
    let cmd = common::wukong_raw_command()
        .arg("completion")
        .arg("fish")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

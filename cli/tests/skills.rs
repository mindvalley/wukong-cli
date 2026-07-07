mod common;

#[test]
fn test_wukong_skills_help() {
    let cmd = common::wukong_raw_command()
        .arg("skills")
        .arg("help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_skills_archive_help() {
    let cmd = common::wukong_raw_command()
        .arg("skills")
        .arg("archive")
        .arg("--help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

#[test]
fn test_wukong_skills_restore_help() {
    let cmd = common::wukong_raw_command()
        .arg("skills")
        .arg("restore")
        .arg("--help")
        .assert()
        .success();

    let output = cmd.get_output();

    insta::assert_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
}

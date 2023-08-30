use assert_cmd::Command;

#[allow(dead_code)]
pub fn wukong_raw_command() -> Command {
    Command::cargo_bin("wukong").unwrap()
}

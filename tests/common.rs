use assert_cmd::Command;

pub fn wukong_raw_command() -> Command {
    Command::cargo_bin("wukong").unwrap()
}

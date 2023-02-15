use snapbox::cmd::Command;

pub fn wukong_raw_command() -> Command {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("wukong"))
}

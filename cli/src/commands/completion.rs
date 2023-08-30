use crate::{error::WKCliError, loader::new_spinner};
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use super::ClapApp;

pub fn handle_completion(shell: Shell) -> Result<bool, WKCliError> {
    let loader = new_spinner();
    loader.set_message("Generating completion ...");

    let mut cmd = ClapApp::command();
    cmd.set_bin_name("wukong");

    generate(shell, &mut cmd, "wukong", &mut std::io::stdout().lock());

    loader.finish_and_clear();

    Ok(true)
}

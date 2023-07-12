use crate::{commands::ClapApp, error::WKError, loader::new_spinner_progress_bar};
use clap::CommandFactory;
use clap_complete::{generate, Shell};

pub fn handle_completion(shell: Shell) -> Result<bool, WKError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Generating completion ...");

    let mut cmd = ClapApp::command();
    cmd.set_bin_name("wukong");

    generate(shell, &mut cmd, "wukong", &mut std::io::stdout().lock());

    progress_bar.finish_and_clear();

    Ok(true)
}

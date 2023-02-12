use crate::{commands::ClapApp, error::CliError, loader::new_spinner_progress_bar, GlobalContext};
use clap::CommandFactory;
use clap_complete::{generate, Shell};

pub fn handle_completion(_context: GlobalContext, shell: Shell) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Generating completion ...");

    let mut cmd = ClapApp::command();
    cmd.set_bin_name("wukong");

    generate(shell, &mut cmd, "wukong", &mut std::io::stdout().lock());

    progress_bar.finish_and_clear();

    Ok(true)
}

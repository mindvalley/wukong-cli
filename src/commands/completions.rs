use crate::{clap_app::ClapApp, error::CliError, GlobalContext};
use clap::CommandFactory;
use clap_complete::{generate, Shell};

pub fn handle_completions<'a>(_context: GlobalContext, shell: Shell) -> Result<bool, CliError<'a>> {
    let mut cmd = ClapApp::command();
    cmd.set_bin_name("wukong");

    generate(shell, &mut cmd, "wukong", &mut std::io::stdout().lock());

    Ok(true)
}

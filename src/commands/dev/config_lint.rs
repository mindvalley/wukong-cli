use crate::{error::CliError, loader::new_spinner_progress_bar};
use elixir_linter::run;
use std::{env::current_dir, path::PathBuf};

pub fn handle_config_lint(path: &PathBuf) -> Result<bool, CliError> {
    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Linting ... ");

    println!("path: {}", path.display());

    let lint_path = match path.try_exists() {
        Ok(true) => {
            if path.to_string_lossy() == "." {
                current_dir().unwrap()
            } else {
                path.to_path_buf()
            }
        }
        Ok(false) => {
            eprintln!("path '{}' does not exist", path.to_string_lossy());
            panic!();
        }
        Err(_) => todo!(),
    };

    run(&lint_path);

    Ok(true)
}

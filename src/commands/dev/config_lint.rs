use crate::{error::CliError, loader::new_spinner_progress_bar};
use elixir_linter::{LintRule, Linter};
use miette::GraphicalReportHandler;
use std::{env::current_dir, path::PathBuf, time::Instant};

pub fn handle_config_lint(path: &PathBuf) -> Result<bool, CliError> {
    let start = Instant::now();

    let progress_bar = new_spinner_progress_bar();
    progress_bar.set_message("Linting ... ");

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

    let linter = Linter::new(LintRule::All);

    let load_time_taken = start.elapsed();

    let output = linter.run(&lint_path);

    let lint_time_taken = start.elapsed() - load_time_taken;

    output.report.iter().for_each(|lint_error| {
        let mut s = String::new();
        GraphicalReportHandler::new()
            .render_report(&mut s, lint_error)
            .unwrap();

        eprintln!("{s}");
    });
    eprintln!(
        "Total time taken: {:?} ({:?} to load, {:?} running {} checks)",
        load_time_taken + lint_time_taken,
        load_time_taken,
        lint_time_taken,
        output.total_checks
    );
    eprintln!("Total files: {}", output.total_file_count);

    Ok(true)
}

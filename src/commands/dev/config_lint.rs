use crate::{error::CliError, loader::new_spinner_progress_bar};
use elixir_linter::{LintRule, Linter};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use miette::{Diagnostic, GraphicalReportHandler};
use rayon::prelude::*;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
    time::Instant,
};

pub fn handle_config_lint(path: &Path) -> Result<bool, CliError> {
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

    let mut overrides = OverrideBuilder::new(&lint_path);
    // overrides.add("**/lib/**/*.{ex,exs}").unwrap();
    // overrides.add("**/test/**/*.{ex,exs}").unwrap();
    overrides.add("**/config/**/*.{ex,exs}").unwrap();

    let mut all_lint_errors = vec![];

    let available_files: Vec<PathBuf> = WalkBuilder::new(&lint_path)
        .overrides(overrides.build().unwrap())
        .build()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    all_lint_errors.par_extend(available_files.par_iter().flat_map(|file| linter.run(file)));

    let lint_time_taken = start.elapsed() - load_time_taken;

    all_lint_errors.iter().for_each(|lint_error| {
        let mut s = String::new();
        println!("{:?}", lint_error.severity());

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
        3
    );
    eprintln!("Total files: {}", available_files.len());

    Ok(true)
}

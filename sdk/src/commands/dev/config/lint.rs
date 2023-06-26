use crate::{error::CliError, loader::new_progress_bar};
// use elixir_linter::{LintRule, Linter};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use indicatif::ParallelProgressIterator;
use miette::GraphicalReportHandler;
use rayon::prelude::*;
use std::{
    env::current_dir,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::Instant,
};

pub fn handle_config_lint(path: &Path) -> Result<bool, CliError> {
    todo!()
    // let start = Instant::now();
    //
    // let lint_path = path.try_exists().map(|value| match value {
    //     true => {
    //         if path.to_string_lossy() == "." {
    //             current_dir()
    //         } else {
    //             Ok(path.to_path_buf())
    //         }
    //     }
    //     false => Err(std::io::Error::new(
    //         ErrorKind::NotFound,
    //         format!("path '{}' does not exist", path.to_string_lossy()),
    //     )),
    // })??;
    //
    // let linter = Linter::new(LintRule::All);
    //
    // let load_time_taken = start.elapsed();
    //
    // let mut overrides = OverrideBuilder::new(&lint_path);
    // overrides.add("**/config/**/*.{ex,exs}").unwrap();
    //
    // let mut all_lint_errors = vec![];
    //
    // let available_files: Vec<PathBuf> = WalkBuilder::new(&lint_path)
    //     .overrides(overrides.build().unwrap())
    //     .build()
    //     .flatten()
    //     .filter(|e| e.path().is_file())
    //     .map(|e| e.path().to_path_buf())
    //     .collect();
    //
    // let progress_bar = new_progress_bar(available_files.len() as u64);
    // progress_bar.set_message("Linting ... ");
    //
    // all_lint_errors.par_extend(
    //     available_files
    //         .par_iter()
    //         .progress_with(progress_bar)
    //         .flat_map(|file| linter.run(file)),
    // );
    //
    // let lint_time_taken = start.elapsed() - load_time_taken;
    //
    // all_lint_errors.iter().for_each(|lint_error| {
    //     let mut s = String::new();
    //
    //     GraphicalReportHandler::new()
    //         .render_report(&mut s, lint_error)
    //         .unwrap();
    //
    //     eprintln!("{s}");
    // });
    // eprintln!(
    //     "Total time taken: {:?} ({:?} to load, {:?} running {} checks)",
    //     load_time_taken + lint_time_taken,
    //     load_time_taken,
    //     lint_time_taken,
    //     3
    // );
    // eprintln!("Total files: {}", available_files.len());
    //
    // if all_lint_errors.is_empty() {
    //     Ok(true)
    // } else {
    //     Ok(false)
    // }
}

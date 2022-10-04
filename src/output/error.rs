use crate::error::CliError;
use owo_colors::{colors::CustomColor, OwoColorize};
use std::error::Error;

pub fn display_error(error: CliError) {
    match error {
        CliError::Io(ref io_error) if io_error.kind() == ::std::io::ErrorKind::BrokenPipe => {
            ::std::process::exit(0);
        }
        _ => {
            eprintln!("{}:", "Error".red());
            eprintln!("\t{}", error);

            //TODO: for --verbose only
            if let Some(source) = error.source() {
                eprintln!("\n{}:", "Caused by".fg::<CustomColor<245, 245, 245>>());
                eprintln!("\t{}", source);
            }

            if let Some(suggestion) = error.suggestion() {
                eprintln!("\n{}:", "Suggestion".cyan());
                eprintln!("\t{}", suggestion);
            }
        }
    };
}

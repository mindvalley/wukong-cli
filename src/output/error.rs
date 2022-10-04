use crate::error::CliError;
use owo_colors::{colors::CustomColor, OwoColorize};
use std::{error::Error, fmt::Display};

pub struct ErrorOutput<'a>(pub CliError<'a>);

impl<'a> Display for ErrorOutput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            CliError::Io(ref io_error) if io_error.kind() == ::std::io::ErrorKind::BrokenPipe => {
                ::std::process::exit(0);
            }
            error => {
                writeln!(f, "{}:", "Error".red())?;
                writeln!(f, "\t{}", error)?;

                //TODO: for --verbose only
                if let Some(source) = error.source() {
                    writeln!(f, "\n{}:", "Caused by".fg::<CustomColor<245, 245, 245>>())?;
                    writeln!(f, "\t{}", source)?;
                }

                if let Some(suggestion) = error.suggestion() {
                    writeln!(f, "\n{}:", "Suggestion".cyan())?;
                    writeln!(f, "\t{}", suggestion)?;
                }
            }
        };

        Ok(())
    }
}

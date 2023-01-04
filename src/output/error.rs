use crate::error::CliError;
use owo_colors::{
    colors::{xterm::Gray, CustomColor},
    OwoColorize,
};
use std::{error::Error, fmt::Display};

pub struct ErrorOutput(pub CliError);

impl Display for ErrorOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            CliError::Io(ref io_error)
                if io_error.kind() == ::std::io::ErrorKind::BrokenPipe
                    || io_error.kind() == ::std::io::ErrorKind::Interrupted =>
            {
                ::std::process::exit(1);
            }
            error => {
                writeln!(f, "{}", error)?;

                //TODO: for --verbose only
                if let Some(source) = error.source() {
                    writeln!(
                        f,
                        "{} {} {}",
                        "Caused by".fg::<CustomColor<245, 245, 245>>(),
                        "-".fg::<Gray>(),
                        source
                    )?;
                }

                if let Some(suggestion) = error.suggestion() {
                    write!(f, "{} {} ", "Suggestion".cyan(), "-".fg::<Gray>())?;

                    if suggestion.lines().count() > 1 {
                        writeln!(f)?;

                        for line in suggestion.lines() {
                            writeln!(f, "\t{}", line)?;
                        }
                    } else {
                        writeln!(f, "{}", suggestion)?;
                    }
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::error::ConfigError;

    use super::*;

    fn error_title_in_red() -> &'static str {
        "\u{1b}[31mError\u{1b}[39m:"
    }

    fn caused_by_title_in_rgb_245_245_245() -> &'static str {
        "\u{1b}[38;2;245;245;245mCaused by\u{1b}[39m:"
    }

    fn suggestion_title_in_cyan() -> &'static str {
        "\u{1b}[36mSuggestion\u{1b}[39m:"
    }

    fn error_output_with_caused_by_and_suggestion(
        error: &str,
        caused_by: &str,
        suggestion: &str,
    ) -> String {
        format!(
            "{}\n\t{}\n\n{}\n\t{}\n\n{}\n\t{}\n",
            error_title_in_red(),
            error,
            caused_by_title_in_rgb_245_245_245(),
            caused_by,
            suggestion_title_in_cyan(),
            suggestion
        )
    }

    fn error_output_with_suggestion(error: &str, suggestion: &str) -> String {
        format!(
            "{}\n\t{}\n\n{}\n\t{}\n",
            error_title_in_red(),
            error,
            suggestion_title_in_cyan(),
            suggestion
        )
    }

    #[test]
    fn test_cli_error_output_format_with_suggestion() {
        let error = CliError::UnInitialised;
        let error_output = ErrorOutput(error);

        assert_eq!(
            format!("{}", error_output),
            error_output_with_suggestion(
                "You are un-initialised.",
                "Run \"wukong init\" to initialise Wukong's configuration before running other commands."
            )
        );
    }

    #[test]
    fn test_cli_error_output_format_with_caused_by() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not exist");
        let error = CliError::ConfigError(ConfigError::NotFound {
            path: "not/path/to/file",
            source: io_error,
        });

        let error_output = ErrorOutput(error);

        assert_eq!(
            format!("{}", error_output),
            error_output_with_caused_by_and_suggestion(
                "Config file not found at \"not/path/to/file\".",
                "File not exist",
                "Run \"wukong init\" to initialise configuration."
            )
        );
    }
}

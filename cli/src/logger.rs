use log::{LevelFilter, Metadata, Record};
use owo_colors::{colors::xterm::Gray, OwoColorize};
use std::{env, str::FromStr};

pub struct Builder {
    max_log_level: log::LevelFilter,
}

#[derive(Debug)]
pub struct Logger;

pub const GLOBAL_LOGGER: &Logger = &Logger;
pub const LOG_LEVEL_ENV: &str = "WUKONG_LOG";

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // metadata.level() <= Level::Info
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                log::Level::Error => "Error".red().to_string(),
                log::Level::Warn => "Warn".yellow().to_string(),
                log::Level::Info => "Info".cyan().to_string(),
                log::Level::Debug => "Debug".blue().to_string(),
                log::Level::Trace => "Trace".fg::<Gray>().to_string(),
            };

            eprintln!("{} {} {}", level, "-".fg::<Gray>(), record.args(),);
        }
    }

    fn flush(&self) {}
}

impl Builder {
    #[must_use = "You must call init() to begin logging"]
    pub fn new() -> Self {
        let default_log_level = match env::var(LOG_LEVEL_ENV) {
            Ok(env_log_level) => {
                // if the WUKONG_LOG environment variable is set wrongly, it will always default to Error level
                LevelFilter::from_str(&env_log_level).unwrap_or(LevelFilter::Error)
            }
            Err(_) => LevelFilter::Error,
        };

        Self {
            max_log_level: default_log_level,
        }
    }

    pub fn with_max_level(mut self, max_log_level: LevelFilter) -> Self {
        // the default log level is Error if the user run command without vebosity flag,
        // so if the level is not Error (the user run command with vebosity flag),
        // we want to override the log level
        if max_log_level != LevelFilter::Error {
            self.max_log_level = max_log_level;
        }

        self
    }

    pub fn init(self) {
        log::set_max_level(self.max_log_level);
        log::set_logger(GLOBAL_LOGGER).expect("unable to init wukong-cli logger");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_max_log_level_without_env_set_should_be_error() {
        // just to be sure the env is clean
        env::remove_var(LOG_LEVEL_ENV);

        let logger_builder = Builder::new();
        assert_eq!(logger_builder.max_log_level, LevelFilter::Error);
    }

    #[test]
    fn test_max_log_level_should_be_trace_with_env_set_to_trace() {
        env::set_var(LOG_LEVEL_ENV, "trace");

        let logger_builder = Builder::new();
        assert_eq!(logger_builder.max_log_level, LevelFilter::Trace);
    }

    #[test]
    fn test_max_log_level_should_be_warn_with_env_set_to_trace_and_override_by_warn_vebosity_flag()
    {
        env::set_var(LOG_LEVEL_ENV, "trace");

        let logger_builder = Builder::new().with_max_level(LevelFilter::Warn);
        assert_eq!(logger_builder.max_log_level, LevelFilter::Warn);
    }
}

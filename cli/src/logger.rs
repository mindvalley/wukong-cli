use log::{LevelFilter, Metadata, Record};
use owo_colors::{colors::xterm::Gray, OwoColorize};
use std::io::Write;
use std::{env, fs::File, str::FromStr, sync::Mutex};

pub struct Builder {
    max_log_level: log::LevelFilter,
    log_file: Mutex<File>,
}

#[derive(Debug)]
pub struct Logger {
    log_file: Mutex<File>,
    max_log_level: log::LevelFilter,
}

impl Logger {
    fn new(log_file: Mutex<File>, max_log_level: log::LevelFilter) -> Self {
        Self {
            log_file,
            max_log_level,
        }
    }
}

pub const LOG_LEVEL_ENV: &str = "WUKONG_LOG";
pub const DEFAULT_LOG_FILE: &str = "wukong-cli.log";

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.max_log_level >= metadata.level()
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

        let log_message = format!(
            "[{}] [{}] [{}] [{}] [line:{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            record.level(),
            record.module_path().expect("Module path not found"),
            record.file().expect("File path not found"),
            record.line().expect("Line number not found"),
            record.args()
        );

        // Open the log file and write the log message
        if let Ok(mut log_file) = self.log_file.lock() {
            if let Err(err) = writeln!(&mut log_file, "{}", log_message) {
                eprintln!("Error writing to log file: {}", err);
            }
        } else {
            eprintln!("Error locking log file for writing.");
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
            log_file: Mutex::new(File::create(DEFAULT_LOG_FILE).unwrap()),
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

    pub fn with_log_file(mut self, log_file: File) -> Self {
        self.log_file = Mutex::new(log_file);
        self
    }

    pub fn init(self) {
        // Set the default log level to debug,
        // To get debug logs regardless of the env log level
        log::set_max_level(LevelFilter::Debug);
        let logger = self.build();
        log::set_boxed_logger(Box::new(logger)).expect("unable to init wukong-cli logger");
    }

    pub fn build(self) -> Logger {
        Logger::new(self.log_file, self.max_log_level)
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

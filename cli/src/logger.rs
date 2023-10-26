use log::{LevelFilter, Metadata, Record};
use once_cell::sync::Lazy;
use owo_colors::{colors::xterm::Gray, OwoColorize};
use std::io::Write;
use std::{env, fs::File, str::FromStr, sync::Mutex};

pub struct Builder {
    max_log_level: log::LevelFilter,
    log_file: Mutex<File>,
    report: bool,
}

#[derive(Debug)]
pub struct Logger {
    log_file: Mutex<File>,
    report: bool,
}

impl Logger {
    fn new(log_file: Mutex<File>, report: bool) -> Self {
        Self { log_file, report }
    }
}

pub const LOG_LEVEL_ENV: &str = "WUKONG_LOG";
pub static DEBUG_LOG_FILE: Lazy<Option<String>> = Lazy::new(|| {
    #[cfg(feature = "prod")]
    return dirs::home_dir().map(|mut path| {
        path.extend([".config", "wukong", "debug_log"]);
        path.to_str().unwrap().to_string()
    });

    #[cfg(not(feature = "prod"))]
    dirs::home_dir().map(|mut path| {
        path.extend([".config", "wukong", "dev", "debug_log"]);
        path.to_str().unwrap().to_string()
    })
});

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = record.level();

        if self.enabled(record.metadata()) {
            let level_with_colors = match level {
                log::Level::Error => "Error".red().to_string(),
                log::Level::Warn => "Warn".yellow().to_string(),
                log::Level::Info => "Info".cyan().to_string(),
                log::Level::Debug => "Debug".blue().to_string(),
                log::Level::Trace => "Trace".fg::<Gray>().to_string(),
            };

            // If report mode is on dont pring the debug logs to the user:
            if !self.report || self.report && level != log::Level::Debug {
                eprintln!(
                    "{} {} {}",
                    level_with_colors,
                    "-".fg::<Gray>(),
                    record.args(),
                );
            }
        }

        let module_path = record.module_path().expect("Module path not found");

        if self.report && level == log::Level::Debug && module_path.starts_with("wukong") {
            let log_message = format!(
                "[{}] [{}] [{}] [{}] [line:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                module_path,
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

        let debug_log_file = DEBUG_LOG_FILE
            .as_ref()
            .expect("Unable to identify user's home directory");

        Self {
            max_log_level: default_log_level,
            log_file: Mutex::new(File::create(debug_log_file).unwrap()),
            report: false,
        }
    }

    pub fn with_report(mut self, report: bool) -> Self {
        if report {
            self.report = report;
        }

        self
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
        if self.report {
            log::set_max_level(LevelFilter::Debug);
        } else {
            log::set_max_level(self.max_log_level);
        }

        let logger = self.build();
        log::set_boxed_logger(Box::new(logger)).expect("unable to init wukong-cli logger");
    }

    pub fn build(self) -> Logger {
        Logger::new(self.log_file, self.report)
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

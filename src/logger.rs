use log::{LevelFilter, Metadata, Record};
use owo_colors::{colors::xterm::Gray, OwoColorize};

pub struct Builder {
    max_log_level: log::LevelFilter,
}

#[derive(Debug)]
pub struct Logger;

pub const GLOBAL_LOGGER: &Logger = &Logger;

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
        Self {
            max_log_level: LevelFilter::Off,
        }
    }

    pub fn with_max_level(mut self, max_log_level: LevelFilter) -> Self {
        self.max_log_level = max_log_level;
        self
    }

    pub fn init(self) {
        log::set_max_level(self.max_log_level);
        log::set_logger(GLOBAL_LOGGER).expect("unable to init wukong-cli logger");
    }
}

use ansi_term::{ANSIString, Colour};
use log::{Level, Metadata, Record};

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
            let open = Colour::Fixed(243).paint("[");
            let level = match record.level() {
                log::Level::Error => Colour::Red.paint("ERROR"),
                log::Level::Warn => Colour::Yellow.paint("WARN"),
                log::Level::Info => Colour::Cyan.paint("INFO"),
                log::Level::Debug => Colour::Blue.paint("DEBUG"),
                log::Level::Trace => Colour::Fixed(245).paint("TRACE"),
            };
            let close = Colour::Fixed(243).paint("]");

            eprintln!(
                "{}{} {}{} {}",
                open,
                level,
                record.target(),
                close,
                record.args(),
            );
        }
    }

    fn flush(&self) {}
}

impl Logger {
    #[must_use = "You must call init() to begin logging"]
    pub fn new() -> Self {
        Self
    }

    pub fn init(self) {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_logger(GLOBAL_LOGGER);
    }
}

use std::io::Write;

pub trait ModuleLog {
    const INFO: bool;
    const WARN: bool;
    const ERROR: bool;
    const DEBUG: bool;
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                log::Level::Error => "ERROR",
                log::Level::Warn => "WARN",
                log::Level::Info => "INFO",
                log::Level::Debug => "DEBUG",
                log::Level::Trace => "TRACE",
            };
            eprintln!("[{}] {}", level, record.args());
        }
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Debug);
}

#[macro_export]
macro_rules! m_info {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::INFO {
            log::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! m_warn {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::WARN {
            log::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! m_error {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::ERROR {
            log::error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! m_debug {
    ($module:ident, $($arg:tt)*) => {
        if <$module as $crate::m_log::ModuleLog>::DEBUG {
            log::debug!($($arg)*);
        }
    };
}
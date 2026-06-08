use std::io::Write;
use std::sync::Once;

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
static GLOBAL_INIT: Once = Once::new();

pub fn global_init() {
    GLOBAL_INIT.call_once(|| {
        let _ = log::set_logger(&LOGGER);
        log::set_max_level(log::LevelFilter::Debug);
    });
}

pub use m_log_macros::define_module;

#[macro_export]
macro_rules! m_info {
    ($module:ident, $($arg:tt)*) => {
        {
            $module::init();
            if <$module as $crate::ModuleLog>::INFO {
                log::info!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! m_warn {
    ($module:ident, $($arg:tt)*) => {
        {
            $module::init();
            if <$module as $crate::ModuleLog>::WARN {
                log::warn!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! m_error {
    ($module:ident, $($arg:tt)*) => {
        {
            $module::init();
            if <$module as $crate::ModuleLog>::ERROR {
                log::error!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! m_debug {
    ($module:ident, $($arg:tt)*) => {
        {
            $module::init();
            if <$module as $crate::ModuleLog>::DEBUG {
                log::debug!($($arg)*);
            }
        }
    };
}
use lazy_static::lazy_static;
use slog::{Drain, Logger, Level, o};
use slog_async;
use slog_term;
use slog_scope;
use slog_stdlog;
use std::env;
use std::sync::Mutex;

lazy_static! {
    pub static ref LOGGER: Logger = {
        let level = get_log_level();
        let decorator = slog_term::TermDecorator::new().stdout().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog::LevelFilter::new(drain, level).fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        Logger::root(drain, o!())
    };
    static ref LOGGER_GUARD: Mutex<Option<slog_scope::GlobalLoggerGuard>> = Mutex::new(None);

}

fn get_log_level() -> Level {
    match env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()).to_lowercase().as_ref() {
        "trace" => Level::Trace,
        "debug" => Level::Debug,
        "info" => Level::Info,
        "warning" => Level::Warning,
        "error" => Level::Error,
        "critical" => Level::Critical,
        _ => Level::Info,
    }
}

pub fn init() {
    let guard = slog_scope::set_global_logger(LOGGER.clone());
    slog_stdlog::init().unwrap();
    let mut guard_store = LOGGER_GUARD.lock().unwrap();
    *guard_store = Some(guard);
}
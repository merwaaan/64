//! Captures `log` crate output into a buffer that the TUI drains into the log panel.

use log::{Level, Log, Metadata, Record};
use std::sync::Mutex;

static LOG_BUF: std::sync::OnceLock<Mutex<Vec<String>>> = std::sync::OnceLock::new();

struct TuiLogger;

impl Log for TuiLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} {}",
            level_prefix(record.level()),
            record.args()
        );
        if let Some(buf) = LOG_BUF.get() {
            if let Ok(mut guard) = buf.lock() {
                guard.push(line);
            }
        }
    }

    fn flush(&self) {}
}

fn level_prefix(level: Level) -> &'static str {
    match level {
        Level::Error => "ERR",
        Level::Warn => "WRN",
        Level::Info => "INF",
        Level::Debug => "DBG",
        Level::Trace => "TRC",
    }
}

/// Initializes the TUI logger. Call once before entering the TUI.
/// All subsequent `log::info!`, `log::warn!`, etc. will be buffered for the log panel.
pub fn init(max_level: log::LevelFilter) {
    LOG_BUF.get_or_init(|| Mutex::new(Vec::new()));
    let _ = log::set_logger(&TuiLogger).map(|()| log::set_max_level(max_level));
}

/// Drains buffered log lines. Call each frame (or when handling events) and append to `App::logs`.
pub fn drain_logs() -> Vec<String> {
    LOG_BUF
        .get()
        .and_then(|buf| buf.lock().ok())
        .map(|mut guard| std::mem::take(&mut *guard))
        .unwrap_or_default()
}

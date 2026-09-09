//! Log bridge: keeps `env_logger` output on stderr, retains a ring buffer of recent records and,
//! once the Tauri app handle is attached, streams every record to the Studio as a `rust-log`
//! event so the Dev Console shows backend diagnostics.

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

pub const RUST_LOG_EVENT: &str = "rust-log";
const HISTORY_CAPACITY: usize = 300;

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct LogRecord {
    pub level: String,
    pub target: String,
    pub message: String,
    /// Milliseconds since the Unix epoch.
    pub timestamp_ms: f64,
}

struct Bridge {
    inner: env_logger::Logger,
    history: Mutex<VecDeque<LogRecord>>,
    app: OnceLock<AppHandle>,
}

static BRIDGE: OnceLock<&'static Bridge> = OnceLock::new();

impl log::Log for Bridge {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if !self.inner.matches(record) {
            return;
        }
        self.inner.log(record);

        // Only our own crates reach the UI; wgpu/tauri internals stay on stderr.
        let target = record.target();
        if !target.starts_with("fxcursor") {
            return;
        }

        let entry = LogRecord {
            level: record.level().to_string().to_lowercase(),
            target: target.to_string(),
            message: record.args().to_string(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64() * 1000.0)
                .unwrap_or(0.0),
        };

        if let Ok(mut h) = self.history.lock() {
            if h.len() >= HISTORY_CAPACITY {
                h.pop_front();
            }
            h.push_back(entry.clone());
        }

        if let Some(app) = self.app.get() {
            let _ = app.emit(RUST_LOG_EVENT, &entry);
        }
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

/// Installs the bridge as the global logger. Call once, before any logging.
pub fn install() {
    let inner = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .build();
    let max_level = inner.filter();
    let bridge: &'static Bridge = Box::leak(Box::new(Bridge {
        inner,
        history: Mutex::new(VecDeque::with_capacity(HISTORY_CAPACITY)),
        app: OnceLock::new(),
    }));
    if BRIDGE.set(bridge).is_ok() && log::set_logger(bridge).is_ok() {
        log::set_max_level(max_level);
    }
}

/// Attach the app handle so subsequent records are streamed to the webview.
pub fn attach(app: &AppHandle) {
    if let Some(bridge) = BRIDGE.get() {
        let _ = bridge.app.set(app.clone());
    }
}

/// Recent backend records (oldest first).
pub fn recent() -> Vec<LogRecord> {
    BRIDGE
        .get()
        .and_then(|b| b.history.lock().ok().map(|h| h.iter().cloned().collect()))
        .unwrap_or_default()
}

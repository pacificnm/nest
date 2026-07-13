//! In-memory ring buffer for GUI log panels.

use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

use crate::level::LogLevel;

static GLOBAL_UI_BUFFER: OnceLock<Arc<LogBuffer>> = OnceLock::new();

/// Ring buffer of recent log lines for in-app viewers (e.g. Kiwi Logs panel).
pub struct LogBuffer {
    lines: Mutex<VecDeque<LogRecord>>,
    capacity: usize,
    total: AtomicUsize,
}

/// One captured log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    /// Severity level.
    pub level: LogLevel,
    /// Tracing target (typically module path).
    pub target: String,
    /// Formatted message body.
    pub message: String,
    /// UTC time formatted as `HH:MM:SS.mmm`.
    pub timestamp: String,
}

impl LogBuffer {
    /// Creates a buffer retaining at most `capacity` lines.
    pub fn new(capacity: usize) -> Arc<Self> {
        let capacity = capacity.max(1);
        Arc::new(Self {
            lines: Mutex::new(VecDeque::with_capacity(capacity.min(256))),
            capacity,
            total: AtomicUsize::new(0),
        })
    }

    /// Appends a log line, evicting the oldest when at capacity.
    pub fn push(&self, record: LogRecord) {
        let mut lines = self.lines.lock().expect("log buffer mutex poisoned");
        if lines.len() >= self.capacity {
            lines.pop_front();
        }
        lines.push_back(record);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns a snapshot of buffered lines in insertion order.
    pub fn snapshot(&self) -> Vec<LogRecord> {
        self.lines
            .lock()
            .expect("log buffer mutex poisoned")
            .iter()
            .cloned()
            .collect()
    }

    /// Clears all buffered lines.
    pub fn clear(&self) {
        self.lines
            .lock()
            .expect("log buffer mutex poisoned")
            .clear();
    }

    /// Number of lines currently held in the buffer.
    pub fn len(&self) -> usize {
        self.lines.lock().expect("log buffer mutex poisoned").len()
    }

    /// Returns true when the buffer contains no lines.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total number of lines ever pushed (including evicted).
    pub fn total_pushed(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }
}

/// Stores the process-global UI log buffer after logging init.
pub fn install_ui_buffer(buffer: Arc<LogBuffer>) {
    let _ = GLOBAL_UI_BUFFER.set(buffer);
}

/// Returns the UI log buffer when the host configured one at startup.
pub fn ui_buffer() -> Option<Arc<LogBuffer>> {
    GLOBAL_UI_BUFFER.get().cloned()
}

pub(crate) fn ui_log_layer(buffer: Arc<LogBuffer>) -> UiLogLayer {
    UiLogLayer { buffer }
}

/// Tracing layer that copies filtered events into a [`LogBuffer`].
pub struct UiLogLayer {
    buffer: Arc<LogBuffer>,
}

impl<S> Layer<S> for UiLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = visitor.finish();
        if message.is_empty() {
            return;
        }

        self.buffer.push(LogRecord {
            level: LogLevel::from_tracing(*meta.level()),
            target: meta.target().to_string(),
            message,
            timestamp: format_utc_time(),
        });
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl MessageVisitor {
    fn finish(self) -> String {
        if let Some(message) = self.message {
            return message;
        }
        if self.fields.is_empty() {
            return String::new();
        }
        self.fields
            .into_iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}").trim_matches('"').to_string());
        } else {
            self.fields
                .push((field.name().to_string(), format!("{value:?}")));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .push((field.name().to_string(), value.to_string()));
        }
    }
}

fn format_utc_time() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_ms = duration.as_millis();
    let ms = total_ms % 1_000;
    let total_secs = total_ms / 1_000;
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = (total_secs / 3_600) % 24;
    format!("{hours:02}:{mins:02}:{secs:02}.{ms:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_evicts_oldest() {
        let buffer = LogBuffer::new(2);
        buffer.push(sample(LogLevel::Info, "first"));
        buffer.push(sample(LogLevel::Info, "second"));
        buffer.push(sample(LogLevel::Info, "third"));

        let lines = buffer.snapshot();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].message, "second");
        assert_eq!(lines[1].message, "third");
        assert_eq!(buffer.total_pushed(), 3);
    }

    #[test]
    fn clear_removes_all_lines() {
        let buffer = LogBuffer::new(8);
        buffer.push(sample(LogLevel::Warn, "warn"));
        buffer.clear();
        assert!(buffer.is_empty());
    }

    fn sample(level: LogLevel, message: &str) -> LogRecord {
        LogRecord {
            level,
            target: "test".into(),
            message: message.into(),
            timestamp: "00:00:00.000".into(),
        }
    }
}

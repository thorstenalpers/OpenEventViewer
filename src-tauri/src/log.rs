use std::collections::VecDeque;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// How much history the buffer keeps. Old entries fall off the front, so a long-running session
/// cannot grow the process without bound.
const CAPACITY: usize = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub timestamp: String,
    pub level: Level,
    /// Where it came from — `import`, `session`, `ui`. Free text, not an enum: a new call site
    /// should not need a schema change to say where it is.
    pub source: String,
    pub message: String,
}

/// An in-memory ring the Log view reads.
///
/// Deliberately not a file: the log exists to answer "what did the app just do", and a log that
/// outlives the run it describes is a second thing to manage, delete and worry about privacy for.
/// Nothing here is written to disk.
#[derive(Debug, Default)]
pub struct Log {
    entries: Mutex<VecDeque<Entry>>,
    /// When off, `Debug` entries are dropped at the door rather than filtered on display.
    debug: Mutex<bool>,
}

impl Log {
    pub fn record(&self, level: Level, source: &str, message: impl Into<String>) {
        if level == Level::Debug && !self.debug_enabled() {
            return;
        }

        let entry = Entry {
            timestamp: timestamp(),
            level,
            source: source.to_string(),
            message: message.into(),
        };

        let mut entries = self.lock();
        if entries.len() == CAPACITY {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.lock().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    pub fn set_debug(&self, enabled: bool) {
        *self
            .debug
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = enabled;
    }

    pub fn debug_enabled(&self) -> bool {
        *self
            .debug
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<Entry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// `HH:MM:SS.mmm` in UTC, derived from the wall clock without pulling in a date library.
///
/// The log is read while the app is running, so the date carries no information the reader does not
/// already have; the milliseconds do, because they are what shows a command hanging.
fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = now.as_secs();
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        (seconds / 3600) % 24,
        (seconds / 60) % 60,
        seconds % 60,
        now.subsec_millis()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_keeps_the_newest_entries_and_drops_the_oldest() {
        let log = Log::default();
        for index in 0..CAPACITY + 50 {
            log.record(Level::Info, "test", format!("entry {index}"));
        }

        let entries = log.entries();
        assert_eq!(entries.len(), CAPACITY);
        assert_eq!(entries.first().expect("first").message, "entry 50");
        assert_eq!(
            entries.last().expect("last").message,
            format!("entry {}", CAPACITY + 49)
        );
    }

    /// Debug entries are dropped when the switch is off, not merely hidden — otherwise the buffer
    /// fills with noise nobody asked for and evicts the entries that matter.
    #[test]
    fn debug_entries_are_refused_unless_the_switch_is_on() {
        let log = Log::default();

        log.record(Level::Debug, "test", "quiet");
        log.record(Level::Info, "test", "kept");
        assert_eq!(log.entries().len(), 1);

        log.set_debug(true);
        log.record(Level::Debug, "test", "loud");
        assert_eq!(log.entries().len(), 2);
        assert_eq!(log.entries()[1].message, "loud");
    }

    #[test]
    fn a_timestamp_is_wall_clock_with_milliseconds() {
        let stamp = timestamp();
        assert_eq!(stamp.len(), 12, "{stamp}");
        assert_eq!(stamp.as_bytes()[2], b':');
        assert_eq!(stamp.as_bytes()[8], b'.');
    }

    #[test]
    fn clearing_leaves_the_buffer_usable() {
        let log = Log::default();
        log.record(Level::Error, "test", "boom");
        log.clear();
        assert!(log.entries().is_empty());

        log.record(Level::Info, "test", "after");
        assert_eq!(log.entries().len(), 1);
    }
}

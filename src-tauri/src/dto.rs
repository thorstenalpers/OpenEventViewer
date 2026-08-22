use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    /// Whether the Log entry appears in the sidebar at all.
    #[serde(default)]
    pub show_logs: bool,
    /// Whether `debug` entries are recorded. Off by default: they are the noisy ones, and a buffer
    /// full of them evicts the entries someone actually went looking for.
    #[serde(default)]
    pub debug_logging: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            show_logs: false,
            debug_logging: false,
        }
    }
}

use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const KEYRING_SERVICE: &str = "com.thorstenalpers.openeventviewer";
const KEYRING_ACCOUNT: &str = "anthropic-api-key";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The local `claude` binary. Nothing leaves the machine that the binary does not already send.
    #[default]
    Cli,
    /// api.anthropic.com, with the key in the Windows Credential Manager.
    Anthropic,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub source: Source,
    pub cli_available: bool,
    pub has_key: bool,
}

pub fn status(source: Source) -> Status {
    Status {
        source,
        cli_available: locate_cli().is_some(),
        has_key: entry().and_then(|e| e.get_password()).is_ok(),
    }
}

fn entry() -> keyring::Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
}

/// Stores the key where this app cannot read it back into the UI — only send it.
pub fn set_key(key: &str) -> AppResult<()> {
    let entry = entry().map_err(|error| AppError::Message(error.to_string()))?;
    if key.trim().is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry
        .set_password(key.trim())
        .map_err(|error| AppError::Message(error.to_string()))
}

/// Two probes because the name resolves differently per machine: an `.exe` runs directly, while a
/// `.cmd` shim does not — `Command::new` will not execute a shim, so that case needs `cmd /C`.
fn locate_cli() -> Option<Vec<String>> {
    let probe = |program: &str, args: &[&str]| {
        crate::quiet(Command::new(program))
            .args(args)
            .arg("--version")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|_| {
                std::iter::once(program.to_string())
                    .chain(args.iter().map(|a| (*a).to_string()))
                    .collect::<Vec<_>>()
            })
    };

    #[cfg(windows)]
    {
        probe("claude", &[]).or_else(|| probe("cmd", &["/C", "claude"]))
    }
    #[cfg(not(windows))]
    {
        probe("claude", &[])
    }
}

use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::assistant;
use crate::dto::Settings;
use crate::error::{AppError, AppResult};
use crate::eventlog::{self, EventRecord, Filter, QueryResult};

pub struct AppState {
    pub config_dir: PathBuf,
    pub log: crate::log::Log,
}

impl AppState {
    fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }
}

/// Runs work that blocks for longer than a frame off the runtime's own threads.
///
/// Tauri answers a synchronous command on the main thread and an `async` one on a runtime worker.
/// Neither suits a query over fifty thousand events: the first freezes the window, the second
/// starves every other command behind it.
async fn blocking<T, F>(work: F) -> AppResult<T>
where
    F: FnOnce() -> AppResult<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?
}

#[tauri::command]
pub async fn events_channels() -> AppResult<Vec<String>> {
    blocking(eventlog::list_channels).await
}

#[tauri::command]
pub async fn events_query(state: State<'_, AppState>, filter: Filter) -> AppResult<QueryResult> {
    let where_from = if filter.channels.is_empty() {
        "System, Application".to_string()
    } else {
        filter.channels.join(", ")
    };
    let result = blocking(move || eventlog::query(&filter)).await?;
    state.log.record(
        crate::log::Level::Info,
        "events",
        format!(
            "{where_from}: {} events in {} ms{}",
            result.events.len(),
            result.elapsed_ms,
            if result.truncated { " (truncated)" } else { "" }
        ),
    );
    Ok(result)
}

#[tauri::command]
pub async fn events_xml(channel: String, record_id: u64) -> AppResult<String> {
    blocking(move || eventlog::event_xml(&channel, record_id)).await
}

/// The exact text a run of events becomes in a prompt.
///
/// The interface asks for it so the preview can show it before anything is sent, rather than
/// building its own version that might differ from the one the host would produce.
#[tauri::command]
pub async fn events_render(events: Vec<EventRecord>) -> AppResult<String> {
    blocking(move || Ok(assistant::render_events_for_prompt(&events))).await
}

/// On its own thread: a hosted model answers in its own time, and the binary is a whole process —
/// neither belongs in a pool slot that every other command is queueing behind.
#[tauri::command]
pub async fn assistant_chat(
    source: assistant::Source,
    messages: Vec<assistant::Message>,
) -> AppResult<String> {
    blocking(move || assistant::chat(source, &messages)).await
}

#[tauri::command(async)]
pub fn log_entries(state: State<'_, AppState>) -> AppResult<Vec<crate::log::Entry>> {
    Ok(state.log.entries())
}

#[tauri::command(async)]
pub fn log_clear(state: State<'_, AppState>) -> AppResult<()> {
    state.log.clear();
    Ok(())
}

/// Lets the interface put its own entries in the host's buffer.
///
/// One buffer rather than two: an error thrown in a view and the command that provoked it belong
/// on the same timeline, and a reader should not have to interleave two logs by hand to see that.
#[tauri::command(async)]
pub fn log_write(
    state: State<'_, AppState>,
    level: crate::log::Level,
    source: String,
    message: String,
) -> AppResult<()> {
    // An error from the interface also goes to stderr, where `npm run start` prints it. The Log
    // view is the better place to read one — but the failure this matters for is the one that
    // leaves no clickable way to get there.
    if matches!(level, crate::log::Level::Error) {
        eprintln!("[{source}] {message}");
    }
    state.log.record(level, &source, message);
    Ok(())
}

/// The generated third-party notices, read from the bundled resource.
///
/// The file ships inside the installer because that is what the licences require — MIT, BSD and ISC
/// all say the notice must accompany the distribution, and a link to a web page does not. Serving it
/// through the bridge is only how the Info page makes it findable; the obligation is met by the file
/// being there at all.
#[tauri::command(async)]
pub fn third_party_licenses(app: AppHandle) -> AppResult<String> {
    const NAME: &str = "THIRD_PARTY_LICENSES.txt";

    let mut tried = Vec::new();
    for directory in [
        app.path().resource_dir().ok(),
        app.path()
            .resource_dir()
            .ok()
            .map(|dir| dir.join("resources")),
        // `tauri dev` runs the binary straight out of target/, where no resource dir is staged.
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")),
    ]
    .into_iter()
    .flatten()
    {
        let candidate = directory.join(NAME);
        if let Ok(text) = std::fs::read_to_string(&candidate) {
            return Ok(text);
        }
        tried.push(candidate);
    }

    Err(AppError::Message(format!(
        "{NAME} is missing — run `npm run licenses`. Looked in: {tried:?}"
    )))
}

#[tauri::command]
pub fn devtools_open(app: AppHandle) -> AppResult<()> {
    #[cfg(debug_assertions)]
    if let Some(window) = app.get_webview_window("main") {
        window.open_devtools();
    }
    let _ = &app;
    Ok(())
}

#[tauri::command(async)]
pub fn assistant_status(source: assistant::Source) -> AppResult<assistant::Status> {
    Ok(assistant::status(source))
}

#[tauri::command(async)]
pub fn assistant_set_key(key: String) -> AppResult<()> {
    assistant::set_key(&key)
}

#[tauri::command(async)]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    let path = state.settings_path();
    match std::fs::read_to_string(&path) {
        Ok(raw) => Ok(serde_json::from_str(&raw).unwrap_or_default()),
        Err(_) => Ok(Settings::default()),
    }
}

#[tauri::command(async)]
pub fn set_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    // The switch has to reach the buffer, not just the file: `record` drops debug entries at the
    // door, so a setting that only lived in settings.json would change nothing until a restart.
    state.log.set_debug(settings.debug_logging);
    state.log.record(
        crate::log::Level::Info,
        "settings",
        format!(
            "saved — theme {}, log view {}, debug {}",
            settings.theme, settings.show_logs, settings.debug_logging
        ),
    );
    std::fs::create_dir_all(&state.config_dir)
        .map_err(|error| AppError::Message(error.to_string()))?;
    std::fs::write(
        state.settings_path(),
        serde_json::to_string_pretty(&settings)?,
    )
    .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(settings)
}

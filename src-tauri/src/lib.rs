mod assistant;
mod commands;
mod dto;
mod error;
mod eventlog;
mod log;

use tauri::Manager;

use commands::AppState;

/// Starts a child process without a console window.
///
/// Every `Command` in this app goes through here. Without the flag Windows gives a console process
/// its own window, and that window takes the input focus for the moment it exists — after which the
/// app's webview no longer has it, and *every* click on it is dropped until something gives it back.
pub fn quiet(mut command: std::process::Command) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let handle = app.handle();
            let config_dir = handle.path().app_config_dir()?;

            // The debug switch lives in settings.json, and the buffer refuses debug entries until
            // it is told — so the stored value has to reach the log before the first command runs.
            let log = log::Log::default();
            if let Ok(raw) = std::fs::read_to_string(config_dir.join("settings.json")) {
                if let Ok(stored) = serde_json::from_str::<dto::Settings>(&raw) {
                    log.set_debug(stored.debug_logging);
                }
            }
            log.record(log::Level::Info, "app", "started");

            app.manage(AppState { config_dir, log });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::events_channels,
            commands::events_query,
            commands::events_xml,
            commands::events_render,
            commands::assistant_chat,
            commands::get_settings,
            commands::set_settings,
            commands::log_entries,
            commands::log_clear,
            commands::log_write,
            commands::third_party_licenses,
            commands::devtools_open,
            commands::assistant_status,
            commands::assistant_set_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenEventViewer");
}

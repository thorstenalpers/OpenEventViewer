mod assistant;
mod catalog;
mod commands;
mod db;
mod deck;
mod dto;
mod error;
mod hub;
mod log;
mod podcast;
mod site;
mod srs;
mod typeset;
mod voice;
mod workshop;

use std::sync::Mutex;

use tauri::webview::WebviewBuilder;
use tauri::window::WindowBuilder;
use tauri::{LogicalPosition, LogicalSize, Manager, WebviewUrl, WindowEvent};

use commands::AppState;

/// Starts a child process without a console window.
///
/// Every `Command` in this app goes through here. Without the flag Windows gives a console process
/// its own window, and that window takes the input focus for the moment it exists — after which the
/// app's webview no longer has it, and *every* click on it is dropped until something gives it back.
/// A dropdown that will not open after a spoken preview is this, not the dropdown.
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let handle = app.handle();
            let data_dir = handle.path().app_local_data_dir()?;
            let config_dir = handle.path().app_config_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let connection = db::open(&data_dir.join("library.sqlite3"))?;
            // The catalog is a second file rather than more tables in the first: it is the part
            // that becomes a server, and a boundary drawn now is one nothing has to be untangled
            // from later.
            let catalog = catalog::open(&data_dir.join("catalog.sqlite3"))?;
            // The debug switch lives in settings.json, and the buffer refuses debug entries until
            // it is told — so the stored value has to reach the log before the first command runs.
            let log = log::Log::default();
            if let Ok(raw) = std::fs::read_to_string(config_dir.join("settings.json")) {
                if let Ok(stored) = serde_json::from_str::<dto::Settings>(&raw) {
                    log.set_debug(stored.debug_logging);
                }
            }
            log.record(log::Level::Info, "app", "started");

            app.manage(AppState {
                connection: Mutex::new(connection),
                catalog: Mutex::new(catalog),
                config_dir,
                data_dir,
                library_search: commands::library_search_paths(handle),
                log,
            });

            // The window carries no webview of its own: the app UI and the learning portal are
            // two children of it, so a foreign page can never share an origin with the UI.
            let window = WindowBuilder::new(app, "main")
                .title(concat!("OpenExamTrainer ", env!("CARGO_PKG_VERSION")))
                .inner_size(1360.0, 900.0)
                .min_inner_size(1000.0, 640.0)
                .build()?;

            let scale = window.scale_factor()?;
            let size = window.inner_size()?.to_logical::<f64>(scale);
            let chrome = window.add_child(
                WebviewBuilder::new(site::CHROME, WebviewUrl::default()),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(size.width, size.height),
            )?;
            // A child webview is not handed the window's input focus. Without this the UI paints
            // and hovers but drops clicks, until a stray mouse event wakes the controller.
            let _ = chrome.set_focus();

            let handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let WindowEvent::Resized(_) = event {
                    site::fit_chrome(&handle);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_binders,
            commands::import_file,
            commands::delete_binder,
            commands::list_questions,
            commands::start_session,
            commands::record_attempt,
            commands::finish_session,
            commands::challenge_results,
            commands::create_project,
            commands::dashboard,
            commands::question_stats,
            commands::question_figure,
            commands::third_party_licenses,
            commands::log_entries,
            commands::log_clear,
            commands::log_write,
            commands::export_deck,
            commands::import_deck,
            commands::peek_deck,
            commands::list_links,
            commands::save_link,
            commands::delete_link,
            commands::list_templates,
            commands::save_template,
            commands::delete_template,
            commands::list_certifications,
            commands::add_certification,
            commands::delete_certification,
            commands::list_progress,
            commands::set_progress,
            commands::timeline,
            commands::list_videos,
            commands::add_video,
            commands::delete_video,
            commands::list_notes,
            commands::save_note,
            commands::site_open,
            commands::site_place,
            commands::site_hide,
            commands::site_history,
            commands::site_url,
            commands::site_focus,
            commands::devtools_open,
            commands::assistant_ask,
            commands::assistant_status,
            commands::assistant_set_key,
            commands::podcast_build,
            commands::notes_summarise,
            commands::notes_podcast,
            commands::list_artefacts,
            commands::delete_artefact,
            commands::notes_pdf,
            commands::voice_packs,
            commands::voice_install,
            commands::voice_cancel,
            commands::voice_remove,
            commands::voice_preview,
            commands::voice_stop,
            commands::voice_warm,
            commands::catalog_identity,
            commands::catalog_rename,
            commands::catalog_list,
            commands::catalog_preview,
            commands::catalog_publish,
            commands::catalog_withdraw,
            commands::catalog_import,
            commands::catalog_rate,
            commands::catalog_ratings,
            commands::catalog_post_result,
            commands::catalog_leaderboard,
            commands::catalog_seeds,
            commands::progress_push,
            commands::progress_pull,
            commands::get_settings,
            commands::set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenExamTrainer");
}

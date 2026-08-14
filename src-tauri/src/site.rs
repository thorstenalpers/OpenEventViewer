use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tauri::webview::WebviewBuilder;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Url, WebviewUrl, Window};

use crate::error::{AppError, AppResult};

pub const CHROME: &str = "chrome";
pub const SITE: &str = "site";

static PARKED: AtomicBool = AtomicBool::new(true);

/// Where the site webview sits inside the window, in logical pixels.
///
/// Measured by the Browse view from its own placeholder rather than computed from constants in
/// two places: the sidebar width lives in CSS, and a second copy in Rust would drift.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn window(app: &AppHandle) -> AppResult<Window> {
    app.get_window("main")
        .ok_or_else(|| AppError::Message("the main window is gone".into()))
}

fn parse(url: &str) -> AppResult<Url> {
    url.parse()
        .map_err(|_| AppError::Message(format!("not a URL: {url}")))
}

/// Navigates the site webview, creating it on first use.
///
/// Nothing is fetched before the user asks for a page: the webview does not exist until this is
/// called, so an app that is only ever used offline never opens a socket.
pub fn open(app: &AppHandle, url: &str, rect: Rect) -> AppResult<()> {
    let target = parse(url)?;

    if let Some(webview) = app.get_webview(SITE) {
        webview
            .navigate(target)
            .map_err(|error| AppError::Message(error.to_string()))?;
        return place(app, rect);
    }

    let window = window(app)?;
    let host = window.clone();
    let handle = app.clone();
    window
        .run_on_main_thread(move || {
            let outcome = host.add_child(
                WebviewBuilder::new(SITE, WebviewUrl::External(target)),
                LogicalPosition::new(rect.x, rect.y),
                LogicalSize::new(rect.width.max(1.0), rect.height.max(1.0)),
            );
            match outcome {
                // A webview that has just been created holds the window's input focus, and a
                // webview without it drops clicks. Handing it straight back is what keeps the
                // toolbar above the portal — back, forward, reload, the address field — alive;
                // clicking into the page below takes the focus the other way.
                Ok(_) => focus_chrome(&handle),
                Err(error) => log(
                    &handle,
                    crate::log::Level::Error,
                    format!("the portal webview could not be created: {error}"),
                ),
            }
        })
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(())
}

/// Moves the window's input focus between the app and the portal.
///
/// Called by the Browse view as the pointer crosses between its toolbar and the page: two webviews
/// share one window, only the focused one receives clicks, and nothing else moves the focus back.
pub fn focus(app: &AppHandle, target: &str) -> AppResult<()> {
    let name = match target {
        "site" => SITE,
        "chrome" => CHROME,
        other => return Err(AppError::Message(format!("no webview called {other}"))),
    };
    if let Some(webview) = app.get_webview(name) {
        let _ = webview.set_focus();
    }
    Ok(())
}

/// The host log, where a failure the user cannot otherwise see is at least findable.
fn log(app: &AppHandle, level: crate::log::Level, message: String) {
    if let Some(state) = app.try_state::<crate::commands::AppState>() {
        state.log.record(level, "site", message);
    }
}

pub fn place(app: &AppHandle, rect: Rect) -> AppResult<()> {
    let Some(webview) = app.get_webview(SITE) else {
        return Ok(());
    };
    let _ = webview.set_size(LogicalSize::new(rect.width.max(1.0), rect.height.max(1.0)));
    let _ = webview.set_position(LogicalPosition::new(rect.x, rect.y));
    PARKED.store(false, Ordering::Relaxed);
    Ok(())
}

/// Parks the site webview off the right edge instead of destroying it, so the logged-in session
/// and the scroll position survive a trip through another view.
pub fn hide(app: &AppHandle) -> AppResult<()> {
    let Some(webview) = app.get_webview(SITE) else {
        return Ok(());
    };
    let window = window(app)?;
    let scale = window
        .scale_factor()
        .map_err(|error| AppError::Message(error.to_string()))?;
    let size = window
        .inner_size()
        .map_err(|error| AppError::Message(error.to_string()))?
        .to_logical::<f64>(scale);
    let _ = webview.set_position(LogicalPosition::new(size.width, 0.0));
    PARKED.store(true, Ordering::Relaxed);
    Ok(())
}

/// Hands the window's input focus back to the app's own webview.
///
/// Called when the portal leaves the screen, and never on a window focus event: a parked site
/// webview keeps the focus it was given, but taking it back on every activation also takes it out
/// of the select popups and the title bar, which is what swallows clicks.
pub fn focus_chrome(app: &AppHandle) {
    if let Some(chrome) = app.get_webview(CHROME) {
        let _ = chrome.set_focus();
    }
}

pub fn eval(app: &AppHandle, script: &str) -> AppResult<()> {
    if let Some(webview) = app.get_webview(SITE) {
        webview
            .eval(script)
            .map_err(|error| AppError::Message(error.to_string()))?;
    }
    Ok(())
}

pub fn current_url(app: &AppHandle) -> AppResult<Option<String>> {
    let Some(webview) = app.get_webview(SITE) else {
        return Ok(None);
    };
    Ok(webview.url().ok().map(|url| url.to_string()))
}

/// Keeps the app's own webview the size of the window. A child webview does not follow its
/// parent, so without this the UI stays at the startup rectangle after the first resize.
pub fn fit_chrome(app: &AppHandle) {
    let Ok(window) = window(app) else { return };
    let Ok(scale) = window.scale_factor() else {
        return;
    };
    let Ok(size) = window.inner_size() else {
        return;
    };
    let size = size.to_logical::<f64>(scale);

    if let Some(chrome) = app.get_webview(CHROME) {
        let _ = chrome.set_position(LogicalPosition::new(0.0, 0.0));
        let _ = chrome.set_size(LogicalSize::new(size.width, size.height));
    }

    // A parked site webview sits at the *old* right edge, so a window that grew slides it back
    // over the UI — invisible there, and swallowing every click that lands on it.
    if PARKED.load(Ordering::Relaxed) {
        let _ = hide(app);
    }
}

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base64::Engine;
use openexamtrainer_ingest::{bank, extract, figures, pdf, vce};
use rusqlite::Connection;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::dto::{
    AttemptResult, Binder, CatalogEntry, Certification, ExamTimeline, Identity, ImportReport,
    LeaderboardRow, Link, Note, QuestionDto, Rating, Session, SessionMode, SessionSummary,
    Settings, SyncReport, Template, UploadPreview, Video,
};
use crate::error::{AppError, AppResult};
use crate::{assistant, catalog, db, deck, podcast, site, voice, workshop};

pub struct AppState {
    pub connection: Mutex<Connection>,
    /// The catalog, which is a second database rather than a server — see `catalog.rs`.
    pub catalog: Mutex<Connection>,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub library_search: Vec<PathBuf>,
    pub log: crate::log::Log,
}

impl AppState {
    fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Always taken after `db()` where a command needs both, so two commands holding one each can
    /// never wait on the other.
    fn catalog(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.catalog
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }
}

/// Derives a certification code from the file name — `certleader-ai900.pdf` is an AI-900 binder.
/// Falls back to the stem so a binder is never labelled with a guess dressed up as a fact.
fn certification_of(stem: &str) -> String {
    let bytes: Vec<char> = stem.to_ascii_uppercase().chars().collect();
    for start in 0..bytes.len() {
        let letters: String = bytes[start..].iter().take(2).collect();
        if letters.len() < 2 || !letters.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        let rest: Vec<char> = bytes[start + 2..]
            .iter()
            .copied()
            .skip_while(|c| *c == '-')
            .take(3)
            .collect();
        if rest.len() == 3 && rest.iter().all(|c| c.is_ascii_digit()) {
            return format!("{letters}-{}", rest.iter().collect::<String>());
        }
    }
    stem.to_string()
}

#[tauri::command(async)]
pub fn list_binders(state: State<'_, AppState>) -> AppResult<Vec<Binder>> {
    db::list_binders(&state.db())
}

/// Where a captured figure lives, keyed by its own content hash so re-importing the same dump
/// rewrites the same bytes instead of accumulating copies.
pub fn figure_path(data_dir: &Path, hash: &str) -> PathBuf {
    data_dir.join("figures").join(format!("{hash}.png"))
}

/// The whole import, free of Tauri so it can be tested against a real file.
///
/// `into` fills a project that was created empty; without it the import makes its own.
pub fn import_into(
    connection: &mut Connection,
    search: &[PathBuf],
    data_dir: &Path,
    source: &Path,
    into: Option<i64>,
) -> AppResult<ImportReport> {
    if source
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("vce"))
    {
        vce::import(source)?;
        return Err(AppError::Message(
            "unreachable: no VCE decoder ships".into(),
        ));
    }

    let file_name = source
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // A published question bank is plain text: no pages, no furniture, no figures to rasterise.
    // Everything after this point — scoring, storage, review, drilling — is the same either way.
    let (report, figures_recovered) = match bank::Format::of(&file_name) {
        Some(format) => (bank::parse(&std::fs::read_to_string(source)?, format)?, 0),
        None => {
            let document = pdf::read_file(source, search)?;
            let mut report = extract(&document);

            // Rasterising the gap above the options recovers the answer areas these dumps draw as
            // images. A question that gets its figure back stops being `needs_source`, so this has
            // to run before the questions are stored.
            let assets = figures::capture(source, &document, &mut report)?;
            if !assets.is_empty() {
                std::fs::create_dir_all(data_dir.join("figures"))?;
                for asset in &assets {
                    std::fs::write(figure_path(data_dir, &asset.hash), &asset.png)?;
                }
            }
            // Questions, not files: two questions that share a figure store one asset, and
            // reporting the file count would understate what was recovered.
            let recovered = report
                .questions
                .iter()
                .filter(|q| !q.figures.is_empty())
                .count();
            (report, recovered)
        }
    };

    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported binder");

    let questions: Vec<QuestionDto> = report.questions.iter().map(QuestionDto::from).collect();
    let binder_id = match into {
        Some(binder_id) => {
            db::fill_project(
                connection,
                binder_id,
                &file_name,
                &report.profile,
                &questions,
            )?;
            binder_id
        }
        None => {
            db::insert_binder(
                connection,
                stem,
                &certification_of(stem),
                &file_name,
                &report.profile,
                &questions,
            )?
            .0
        }
    };

    let binder = db::binder(connection, binder_id)?
        .ok_or_else(|| AppError::Message("the project vanished right after insert".into()))?;

    Ok(ImportReport {
        binder,
        profile: report.profile,
        pages: report.pages,
        furniture_dropped: report.furniture_dropped,
        missing_numbers: report.missing_numbers,
        stub_markers: report.stub_markers,
        figures_recovered,
    })
}

#[tauri::command(async)]
pub fn create_project(
    state: State<'_, AppState>,
    title: String,
    certification: String,
    doc_url: Option<String>,
) -> AppResult<Binder> {
    let mut connection = state.db();
    let doc_url = doc_url.unwrap_or_default();
    let id = db::create_project(
        &mut connection,
        title.trim(),
        certification.trim(),
        doc_url.trim(),
    )?;
    db::binder(&connection, id)?
        .ok_or_else(|| AppError::Message("the project vanished right after insert".into()))
}

#[tauri::command(async)]
pub fn dashboard(state: State<'_, AppState>) -> AppResult<db::Dashboard> {
    db::dashboard(&state.db())
}

#[tauri::command(async)]
pub fn import_file(
    state: State<'_, AppState>,
    path: String,
    project_id: Option<i64>,
) -> AppResult<ImportReport> {
    state
        .log
        .record(crate::log::Level::Info, "import", format!("reading {path}"));
    let mut connection = state.db();
    let result = import_into(
        &mut connection,
        &state.library_search,
        &state.data_dir,
        Path::new(&path),
        project_id,
    );
    match &result {
        Ok(report) => state.log.record(
            crate::log::Level::Info,
            "import",
            format!(
                "{} questions, {} figures, profile {}",
                report.binder.question_count, report.figures_recovered, report.profile
            ),
        ),
        Err(error) => state
            .log
            .record(crate::log::Level::Error, "import", error.to_string()),
    }
    result
}

/// A captured figure as a `data:` URL. The bytes go through the bridge rather than the asset
/// protocol so the UI keeps its single, scoped channel to the host.
///
/// The hash is checked before it ever reaches the filesystem: it arrives from the webview, and
/// `figure_path` would happily join `../../` into somewhere else on the disk.
pub fn read_figure(data_dir: &Path, hash: &str) -> AppResult<String> {
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Message(format!("not a content hash: {hash}")));
    }
    let png = std::fs::read(figure_path(data_dir, hash))?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    ))
}

#[tauri::command(async)]
pub fn question_figure(state: State<'_, AppState>, hash: String) -> AppResult<String> {
    read_figure(&state.data_dir, &hash)
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

#[tauri::command(async)]
pub fn delete_binder(state: State<'_, AppState>, binder_id: i64) -> AppResult<()> {
    db::delete_binder(&state.db(), binder_id)
}

#[tauri::command(async)]
pub fn list_questions(
    state: State<'_, AppState>,
    binder_id: i64,
    only_review: Option<bool>,
) -> AppResult<Vec<QuestionDto>> {
    db::list_questions(&state.db(), binder_id, only_review.unwrap_or(false))
}

#[tauri::command(async)]
pub fn start_session(
    state: State<'_, AppState>,
    binder_id: i64,
    mode: SessionMode,
    source_session_id: Option<i64>,
    rules: Option<db::RuleSet>,
) -> AppResult<Session> {
    let rules = rules.unwrap_or_default();
    let connection = state.db();
    let binder = db::binder(&connection, binder_id)?
        .ok_or_else(|| AppError::Message(format!("no binder {binder_id}")))?;
    let questions = db::session_questions(&connection, binder_id, mode, source_session_id, &rules)?;
    let id = db::create_session(&connection, binder_id, mode, &rules)?;

    Ok(Session {
        id,
        binder_id,
        binder_title: binder.title,
        mode,
        rules,
        questions,
    })
}

#[tauri::command(async)]
pub fn question_stats(
    state: State<'_, AppState>,
    binder_id: i64,
) -> AppResult<Vec<db::QuestionStat>> {
    db::question_stats(&state.db(), binder_id)
}

#[tauri::command(async)]
pub fn challenge_results(
    state: State<'_, AppState>,
    binder_id: i64,
    seed: i64,
) -> AppResult<Vec<db::ChallengeResult>> {
    db::challenge_results(&state.db(), binder_id, seed)
}

#[tauri::command(async)]
pub fn record_attempt(
    state: State<'_, AppState>,
    session_id: i64,
    question_id: i64,
    given: Vec<String>,
    elapsed_ms: i64,
) -> AppResult<AttemptResult> {
    let connection = state.db();
    let questions = {
        let mut statement = connection.prepare("SELECT payload FROM questions WHERE id = ?1")?;
        let rows = statement
            .query_map([question_id], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };
    let payload = questions
        .first()
        .ok_or_else(|| AppError::Message(format!("no question {question_id}")))?;
    let question: QuestionDto = serde_json::from_str(payload)?;

    let mut expected: Vec<char> = question.answer_letters.clone();
    expected.sort_unstable();
    let mut actual: Vec<char> = given.iter().filter_map(|s| s.chars().next()).collect();
    actual.sort_unstable();
    let correct = expected == actual;

    db::insert_attempt(
        &connection,
        session_id,
        question_id,
        &given.join(""),
        correct,
        elapsed_ms,
    )?;
    db::reschedule(&connection, question_id, correct)?;

    Ok(AttemptResult {
        correct,
        answer_letters: question.answer_letters,
    })
}

#[tauri::command(async)]
pub fn finish_session(state: State<'_, AppState>, session_id: i64) -> AppResult<SessionSummary> {
    let totals = db::finish_session(&state.db(), session_id)?;
    Ok(SessionSummary {
        session_id,
        binder_id: totals.binder_id,
        mode: totals.mode,
        total: totals.total,
        correct: totals.correct,
        elapsed_ms: totals.elapsed_ms,
        wrong_question_ids: totals.wrong_question_ids,
    })
}

#[tauri::command(async)]
pub fn export_deck(
    state: State<'_, AppState>,
    binder_id: i64,
    path: String,
) -> AppResult<deck::Manifest> {
    deck::export(&state.db(), binder_id, &state.data_dir, Path::new(&path))
}

#[tauri::command(async)]
pub fn import_deck(state: State<'_, AppState>, path: String) -> AppResult<Binder> {
    let mut connection = state.db();
    deck::import(&mut connection, &state.data_dir, Path::new(&path))
}

#[tauri::command(async)]
pub fn list_links(state: State<'_, AppState>, binder_id: i64) -> AppResult<Vec<Link>> {
    db::list_links(&state.db(), binder_id)
}

#[tauri::command(async)]
pub fn list_videos(state: State<'_, AppState>, binder_id: i64) -> AppResult<Vec<Video>> {
    db::list_videos(&state.db(), binder_id)
}

#[tauri::command(async)]
pub fn add_video(
    state: State<'_, AppState>,
    binder_id: i64,
    video: Video,
) -> AppResult<Vec<Video>> {
    let connection = state.db();
    db::insert_video(&connection, binder_id, &video)?;
    db::list_videos(&connection, binder_id)
}

#[tauri::command(async)]
pub fn delete_video(
    state: State<'_, AppState>,
    binder_id: i64,
    video_id: i64,
) -> AppResult<Vec<Video>> {
    let connection = state.db();
    db::delete_video(&connection, video_id)?;
    db::list_videos(&connection, binder_id)
}

#[tauri::command(async)]
pub fn list_notes(state: State<'_, AppState>, binder_id: i64) -> AppResult<Vec<Note>> {
    db::list_notes(&state.db(), binder_id)
}

#[tauri::command(async)]
pub fn save_note(state: State<'_, AppState>, binder_id: i64, note: Note) -> AppResult<Vec<Note>> {
    let connection = state.db();
    db::upsert_note(&connection, binder_id, &note)?;
    db::list_notes(&connection, binder_id)
}

#[tauri::command(async)]
pub fn peek_deck(path: String) -> AppResult<deck::Manifest> {
    deck::peek(Path::new(&path))
}

#[tauri::command]
pub fn site_open(app: AppHandle, url: String, rect: site::Rect) -> AppResult<()> {
    site::open(&app, &url, rect)
}

#[tauri::command]
pub fn site_place(app: AppHandle, rect: site::Rect) -> AppResult<()> {
    site::place(&app, rect)
}

#[tauri::command]
pub fn site_hide(app: AppHandle) -> AppResult<()> {
    site::hide(&app)?;
    site::focus_chrome(&app);
    Ok(())
}

/// Opens the developer tools on the app's own webview.
///
/// Driven from a key handler rather than left to WebView2's F12: the UI is a *child* webview
/// created through `add_child`, and a child does not reliably receive the browser accelerators.
/// Without this there is no way to read a console error out of the running app.
/// Debug builds only. Tauri gates `open_devtools` behind `debug_assertions` or its own `devtools`
/// feature, and turning that feature on drags in a combination that tauri 2.11.5 does not compile:
/// its own `Window::menu()` call stops resolving. A release build therefore has no console, which
/// is the trade the broken build forces rather than one worth arguing for.
/// Hands the input focus to one of the two webviews. The Browse view calls it as the pointer
/// crosses between its toolbar and the portal below.
#[tauri::command]
pub fn site_focus(app: AppHandle, target: String) -> AppResult<()> {
    site::focus(&app, &target)
}

#[tauri::command]
pub fn devtools_open(app: AppHandle) -> AppResult<()> {
    #[cfg(debug_assertions)]
    if let Some(webview) = app.get_webview(site::CHROME) {
        webview.open_devtools();
    }
    let _ = &app;
    Ok(())
}

#[tauri::command]
pub fn site_history(app: AppHandle, step: i32) -> AppResult<()> {
    let script = match step {
        s if s < 0 => "history.back()",
        s if s > 0 => "history.forward()",
        _ => "location.reload()",
    };
    site::eval(&app, script)
}

#[tauri::command]
pub fn site_url(app: AppHandle) -> AppResult<Option<String>> {
    site::current_url(&app)
}

#[tauri::command(async)]
pub fn assistant_status(source: assistant::Source) -> AppResult<assistant::Status> {
    Ok(assistant::status(source))
}

#[tauri::command(async)]
pub fn assistant_set_key(key: String) -> AppResult<()> {
    assistant::set_key(&key)
}

/// Asks the assistant about one question.
///
/// On its own thread: a hosted model answers in its own time, and the binary is a whole process —
/// neither belongs in a pool slot that every other command is queueing behind.
#[tauri::command]
pub async fn assistant_ask(
    app: AppHandle,
    source: assistant::Source,
    task: assistant::Task,
    question_id: i64,
) -> AppResult<String> {
    blocking(move || {
        let state = app.state::<AppState>();
        let question = {
            let connection = state.db();
            let mut statement =
                connection.prepare("SELECT id, payload FROM questions WHERE id = ?1")?;
            let rows = statement
                .query_map([question_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let (id, payload) = rows
                .into_iter()
                .next()
                .ok_or_else(|| AppError::Message(format!("no question {question_id}")))?;
            let mut dto: QuestionDto = serde_json::from_str(&payload)?;
            dto.id = id;
            dto
        };

        assistant::ask(source, &assistant::build_prompt(task, &question))
    })
    .await
}

#[tauri::command]
pub async fn podcast_build(
    app: AppHandle,
    binder_id: i64,
    question_ids: Vec<i64>,
    options: podcast::Options,
) -> AppResult<podcast::Episode> {
    blocking(move || {
        let state = app.state::<AppState>();
        let (binder, questions) = {
            let connection = state.db();
            let binder = db::binder(&connection, binder_id)?
                .ok_or_else(|| AppError::Message(format!("no binder {binder_id}")))?;
            let all = db::list_questions(&connection, binder_id, false)?;
            let selected: Vec<QuestionDto> = if question_ids.is_empty() {
                all.into_iter().filter(|q| !q.needs_source).collect()
            } else {
                all.into_iter()
                    .filter(|q| question_ids.contains(&q.id))
                    .collect()
            };
            (binder, selected)
        };

        let destination =
            podcast::default_destination(&state.data_dir, &binder.title, options.format);
        podcast::generate(&questions, &options, &state.data_dir, &destination)
    })
    .await
}

#[tauri::command(async)]
pub fn list_templates(state: State<'_, AppState>) -> AppResult<Vec<Template>> {
    db::list_templates(&state.db())
}

#[tauri::command(async)]
pub fn save_template(
    state: State<'_, AppState>,
    name: String,
    doc_url: String,
) -> AppResult<Vec<Template>> {
    let connection = state.db();
    db::save_template(&connection, name.trim(), doc_url.trim())?;
    db::list_templates(&connection)
}

#[tauri::command(async)]
pub fn delete_template(state: State<'_, AppState>, template_id: i64) -> AppResult<Vec<Template>> {
    let connection = state.db();
    db::delete_template(&connection, template_id)?;
    db::list_templates(&connection)
}

#[tauri::command(async)]
pub fn list_certifications(
    state: State<'_, AppState>,
    binder_id: i64,
) -> AppResult<Vec<Certification>> {
    db::list_certifications(&state.db(), binder_id)
}

#[tauri::command(async)]
pub fn add_certification(
    state: State<'_, AppState>,
    binder_id: i64,
    passed_at: String,
    note: String,
) -> AppResult<Vec<Certification>> {
    let connection = state.db();
    db::add_certification(&connection, binder_id, passed_at.trim(), note.trim())?;
    db::list_certifications(&connection, binder_id)
}

#[tauri::command(async)]
pub fn delete_certification(
    state: State<'_, AppState>,
    binder_id: i64,
    certification_id: i64,
) -> AppResult<Vec<Certification>> {
    let connection = state.db();
    db::delete_certification(&connection, binder_id, certification_id)?;
    db::list_certifications(&connection, binder_id)
}

#[tauri::command(async)]
pub fn list_progress(state: State<'_, AppState>, binder_id: i64) -> AppResult<Vec<String>> {
    db::list_progress(&state.db(), binder_id)
}

#[tauri::command(async)]
pub fn set_progress(
    state: State<'_, AppState>,
    binder_id: i64,
    step: String,
    done: bool,
) -> AppResult<Vec<String>> {
    let connection = state.db();
    db::set_progress(&connection, binder_id, &step, done)?;
    db::list_progress(&connection, binder_id)
}

#[tauri::command(async)]
pub fn timeline(state: State<'_, AppState>) -> AppResult<Vec<ExamTimeline>> {
    db::timeline(&state.db())
}

#[tauri::command(async)]
pub fn save_link(state: State<'_, AppState>, binder_id: i64, link: Link) -> AppResult<Vec<Link>> {
    let connection = state.db();
    db::save_link(&connection, binder_id, &link)?;
    db::list_links(&connection, binder_id)
}

#[tauri::command(async)]
pub fn delete_link(
    state: State<'_, AppState>,
    binder_id: i64,
    link_id: i64,
) -> AppResult<Vec<Link>> {
    let connection = state.db();
    db::delete_link(&connection, binder_id, link_id)?;
    db::list_links(&connection, binder_id)
}

#[tauri::command(async)]
pub fn voice_packs(state: State<'_, AppState>) -> AppResult<Vec<voice::PackInfo>> {
    voice::list(&state.data_dir)
}

/// Downloads and unpacks a voice pack. How far it has come arrives as `voice:progress` events while
/// this call is still running.
///
/// `spawn_blocking`, not `command(async)`: the latter runs the body *inside* the async runtime, and
/// the blocking HTTP client tears down its own runtime there, which panics before the first byte.
#[tauri::command]
pub async fn voice_install(app: AppHandle, id: String) -> AppResult<voice::PackInfo> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let data_dir = state.data_dir.clone();
        state
            .log
            .record(crate::log::Level::Info, "voice", format!("{id}: fetching"));

        let report = |progress: voice::Progress| {
            let _ = app.emit("voice:progress", progress);
        };
        let outcome = voice::install(&data_dir, &id, &report);

        // Minutes of download and then an unpack: whichever way it ends, the log is where someone
        // looks when the settings page says something they did not expect.
        match &outcome {
            Ok(pack) => state.log.record(
                crate::log::Level::Info,
                "voice",
                format!("{id}: installed, {} voices", pack.voices),
            ),
            Err(error) => {
                state
                    .log
                    .record(crate::log::Level::Error, "voice", format!("{id}: {error}"))
            }
        }
        outcome
    })
    .await
    .map_err(|error| AppError::Message(error.to_string()))?
}

/// Stops one running download at its next chunk.
#[tauri::command]
pub fn voice_cancel(id: String) {
    voice::cancel(&id);
}

#[tauri::command(async)]
pub fn voice_remove(state: State<'_, AppState>, id: String) -> AppResult<()> {
    voice::remove(&state.data_dir, &id)
}

/// Reads one sentence out loud, so a voice can be heard before an episode is recorded with it.
///
/// An empty id is the Windows voice for the language, which is what an episode gets when no pack is
/// chosen — previewing only the downloaded ones would leave the default the one thing never heard.
///
/// On a thread of its own, not in the async pool: this call lasts as long as the sentence does, and
/// a pool worker held for four seconds is four seconds in which the stop button waits its turn.
#[tauri::command]
pub async fn voice_preview(
    app: AppHandle,
    id: String,
    speaker: i32,
    text: String,
    language: podcast::Language,
) -> AppResult<()> {
    blocking(move || {
        let data_dir = app.state::<AppState>().data_dir.clone();
        if id.is_empty() {
            return podcast::preview(&data_dir, language, &text);
        }
        voice::speak(&data_dir, &id, speaker, &text)
    })
    .await
}

/// Runs work that blocks for longer than a frame off the runtime's own threads.
///
/// Tauri answers a synchronous command on the main thread and an `async` one on a runtime worker.
/// Neither suits minutes of synthesis or a spoken sentence: the first freezes the window, the
/// second starves every other command behind it.
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
pub fn voice_stop() {
    voice::silence();
}

/// Loads a voice's model ahead of the first preview. Fired when one is chosen, so the wait lands
/// where nobody is listening for it.
#[tauri::command]
pub async fn voice_warm(app: AppHandle, id: String) -> AppResult<()> {
    blocking(move || {
        let data_dir = app.state::<AppState>().data_dir.clone();
        voice::warm(&data_dir, &id)
    })
    .await
}

/// Turns this project's notes into a study summary and stores it beside the project.
///
/// `command(async)` because the assistant can take half a minute, and a blocked main thread is a
/// frozen window.
#[tauri::command]
pub async fn notes_summarise(
    app: AppHandle,
    binder_id: i64,
    source: assistant::Source,
) -> AppResult<Vec<workshop::Artefact>> {
    blocking(move || {
        let state = app.state::<AppState>();
        let (title, notes, stamp) = {
            let connection = state.db();
            let binder = db::binder(&connection, binder_id)?
                .ok_or_else(|| AppError::Message(format!("no binder {binder_id}")))?;
            let notes = db::list_notes(&connection, binder_id)?;
            // SQLite owns the clock everywhere else in this app; the file name follows it rather than
            // introducing a second one that drifts.
            let stamp = db::now(&connection)?.replace([':', ' ', '-'], "");
            (binder.title, notes, stamp)
        };

        state.log.record(
            crate::log::Level::Info,
            "workshop",
            format!("summarising {} notes for {title}", notes.len()),
        );
        let made = workshop::summarise(&state.data_dir, binder_id, &title, &notes, source, &stamp);
        match &made {
            Ok(artefact) => state.log.record(
                crate::log::Level::Info,
                "workshop",
                format!("wrote {}", artefact.name),
            ),
            Err(error) => state
                .log
                .record(crate::log::Level::Error, "workshop", error.to_string()),
        }
        made?;
        workshop::list(&state.data_dir, binder_id)
    })
    .await
}

/// Reads one artefact — a summary, normally — as a podcast episode stored next to it.
#[tauri::command]
pub async fn notes_podcast(
    app: AppHandle,
    binder_id: i64,
    name: String,
    options: podcast::Options,
) -> AppResult<Vec<workshop::Artefact>> {
    blocking(move || {
        let state = app.state::<AppState>();
        let title = {
            let connection = state.db();
            db::binder(&connection, binder_id)?
                .ok_or_else(|| AppError::Message(format!("no binder {binder_id}")))?
                .title
        };

        let source = workshop::path_of(&state.data_dir, binder_id, &name)?;
        let text = std::fs::read_to_string(&source)
            .map_err(|error| AppError::Message(format!("{name}: {error}")))?;
        let destination = source.with_extension(options.format.extension());

        podcast::generate_document(&text, &title, &options, &state.data_dir, &destination)?;
        workshop::list(&state.data_dir, binder_id)
    })
    .await
}

#[tauri::command(async)]
pub fn list_artefacts(
    state: State<'_, AppState>,
    binder_id: i64,
) -> AppResult<Vec<workshop::Artefact>> {
    workshop::list(&state.data_dir, binder_id)
}

#[tauri::command(async)]
pub fn delete_artefact(
    state: State<'_, AppState>,
    binder_id: i64,
    name: String,
) -> AppResult<Vec<workshop::Artefact>> {
    workshop::delete(&state.data_dir, binder_id, &name)
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

#[tauri::command(async)]
pub fn catalog_identity(state: State<'_, AppState>) -> AppResult<Identity> {
    catalog::identity(&state.catalog())
}

#[tauri::command(async)]
pub fn catalog_rename(state: State<'_, AppState>, name: String) -> AppResult<Identity> {
    catalog::rename(&state.catalog(), &name)
}

#[tauri::command(async)]
pub fn catalog_list(
    state: State<'_, AppState>,
    filter: Option<catalog::Filter>,
) -> AppResult<Vec<CatalogEntry>> {
    catalog::list(&state.catalog(), &filter.unwrap_or_default())
}

/// What publishing would put in the catalog. Builds the real deck, measures it, and throws it away
/// — a preview counted off the tables instead would be a preview of a different file.
#[tauri::command(async)]
pub fn catalog_preview(state: State<'_, AppState>, binder_id: i64) -> AppResult<UploadPreview> {
    let library = state.db();
    let binder = db::binder(&library, binder_id)?
        .ok_or_else(|| AppError::Message(format!("no binder {binder_id}")))?;
    let questions = db::list_questions(&library, binder_id, false)?;
    let mut figures: Vec<&String> = questions.iter().flat_map(|q| &q.figures).collect();
    figures.sort();
    figures.dedup();

    let scratch = catalog::deck_path(&state.data_dir, &format!("{binder_id}.preview"));
    if let Some(parent) = scratch.parent() {
        std::fs::create_dir_all(parent)?;
    }
    deck::export(&library, binder_id, &state.data_dir, &scratch)?;
    let bytes = std::fs::metadata(&scratch)?.len() as i64;
    std::fs::remove_file(&scratch).ok();

    Ok(catalog::preview(
        &binder.title,
        &binder.certification,
        &catalog::Contents {
            questions: questions.len(),
            links: db::list_links(&library, binder_id)?.len(),
            videos: db::list_videos(&library, binder_id)?.len(),
            notes: db::list_notes(&library, binder_id)?.len(),
            figures: figures.len(),
            bytes,
        },
    ))
}

#[tauri::command(async)]
pub fn catalog_publish(state: State<'_, AppState>, binder_id: i64) -> AppResult<CatalogEntry> {
    let library = state.db();
    let catalog = state.catalog();

    let binder = db::binder(&library, binder_id)?
        .ok_or_else(|| AppError::Message(format!("no binder {binder_id}")))?;
    if binder.question_count == 0 {
        return Err(AppError::Message(
            "an empty project has nothing to publish — import a file into it first".into(),
        ));
    }

    let entry_id = match db::remote_id(&library, binder_id)? {
        Some(existing) => existing,
        None => catalog::new_entry_id(&catalog)?,
    };
    let path = catalog::deck_path(&state.data_dir, &catalog::storage_path(&entry_id));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    deck::export(&library, binder_id, &state.data_dir, &path)?;
    let bytes = std::fs::metadata(&path)?.len() as i64;

    catalog::publish(
        &catalog,
        &catalog::Publication {
            entry_id: &entry_id,
            title: &binder.title,
            certification: &binder.certification,
            profile: &binder.profile,
            question_count: binder.question_count,
            needs_source_count: binder.needs_source_count,
            bytes,
        },
    )?;
    db::set_remote_id(&library, binder_id, Some(&entry_id))?;
    state.log.record(
        crate::log::Level::Info,
        "catalog",
        format!("published {} — {bytes} bytes", binder.title),
    );

    catalog::entry(&catalog, &entry_id)
}

#[tauri::command(async)]
pub fn catalog_withdraw(
    state: State<'_, AppState>,
    entry_id: String,
) -> AppResult<Vec<CatalogEntry>> {
    let library = state.db();
    let catalog = state.catalog();
    catalog::withdraw(&catalog, &state.data_dir, &entry_id)?;
    // The binder keeps no pointer to an entry that is gone, so publishing it again is a new entry
    // rather than a resurrection of the one somebody may already have rated.
    for (remote_id, binder_id) in db::published_binders(&library)? {
        if remote_id == entry_id {
            db::set_remote_id(&library, binder_id, None)?;
        }
    }
    catalog::list(&catalog, &catalog::Filter::default())
}

#[tauri::command(async)]
pub fn catalog_import(state: State<'_, AppState>, entry_id: String) -> AppResult<Binder> {
    let mut library = state.db();
    let catalog = state.catalog();
    let entry = catalog::entry(&catalog, &entry_id)?;
    let path = catalog::deck_path(&state.data_dir, &catalog::storage_path(&entry_id));
    state.log.record(
        crate::log::Level::Info,
        "catalog",
        format!("importing {}", entry.title),
    );
    deck::import(&mut library, &state.data_dir, &path)
}

#[tauri::command(async)]
pub fn catalog_rate(
    state: State<'_, AppState>,
    entry_id: String,
    stars: i64,
    comment: String,
) -> AppResult<Vec<Rating>> {
    catalog::rate(&state.catalog(), &entry_id, stars, &comment)
}

#[tauri::command(async)]
pub fn catalog_ratings(state: State<'_, AppState>, entry_id: String) -> AppResult<Vec<Rating>> {
    catalog::ratings(&state.catalog(), &entry_id)
}

#[tauri::command(async)]
pub fn catalog_post_result(
    state: State<'_, AppState>,
    entry_id: String,
    session_id: i64,
) -> AppResult<Vec<LeaderboardRow>> {
    let library = state.db();
    let catalog = state.catalog();
    let result = db::session_result(&library, session_id)?;
    catalog::post_result(
        &catalog,
        &entry_id,
        result.seed,
        result.total,
        result.correct,
        result.elapsed_ms,
    )
}

#[tauri::command(async)]
pub fn catalog_leaderboard(
    state: State<'_, AppState>,
    entry_id: String,
    seed: i64,
) -> AppResult<Vec<LeaderboardRow>> {
    catalog::leaderboard(&state.catalog(), &entry_id, seed)
}

#[tauri::command(async)]
pub fn catalog_seeds(state: State<'_, AppState>, entry_id: String) -> AppResult<Vec<i64>> {
    catalog::seeds(&state.catalog(), &entry_id)
}

#[tauri::command(async)]
pub fn progress_push(state: State<'_, AppState>) -> AppResult<SyncReport> {
    let library = state.db();
    let catalog = state.catalog();
    catalog::push(&catalog, &library)
}

#[tauri::command(async)]
pub fn progress_pull(state: State<'_, AppState>) -> AppResult<SyncReport> {
    let library = state.db();
    let catalog = state.catalog();
    catalog::pull(&catalog, &library)
}

/// Where to look for `pdfium.dll`: next to the executable, in the bundled resource directory, and
/// — for `tauri dev` — in the repository's `vendor/` copy.
pub fn library_search_paths(app: &AppHandle) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(directory) = exe.parent() {
            paths.push(directory.to_path_buf());
        }
    }
    if let Ok(resources) = app.path().resource_dir() {
        paths.push(resources);
    }
    paths.push(pdf::vendored_library_dir());
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_certification_code_is_read_out_of_the_file_name() {
        assert_eq!(certification_of("certleader-ai900"), "AI-900");
        assert_eq!(
            certification_of("Microsoft.PracticeTest.AZ-900.v2021"),
            "AZ-900"
        );
        assert_eq!(certification_of("my own notes"), "my own notes");
    }

    /// The second import path: a published question bank is a text file, and everything after the
    /// parse — storage, scoring, drilling — is the same as for a PDF.
    #[test]
    fn a_markdown_question_bank_imports_into_a_drillable_binder() {
        let source = "<h5>1. Which service trains models?</h5>\n<ol type='a'>\n  <li>Azure \
                      Machine Learning</li>\n  <li>Azure Bot Service</li>\n</ol>\n<details>\n  \
                      <summary>Show Answer</summary>\n  <p>Azure Machine Learning</p>\n\
                      </details>\n\n<h5>2. Which two read images?</h5>\n<ol type='a'>\n  \
                      <li>OCR</li>\n  <li>Sentiment analysis</li>\n  <li>Object detection</li>\n\
                      </ol>\n<details>\n  <summary>Show Answer</summary>\n  <p>['OCR', 'Object \
                      detection']</p>\n</details>";

        let dir = std::env::temp_dir().join("oet-bank-import");
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("AI-900 practice questions.md");
        std::fs::write(&path, source).expect("write");

        let mut connection = db::open_in_memory().expect("schema");
        let report = import_into(&mut connection, &[], &dir, &path, None).expect("import");

        assert_eq!(report.profile, "bank-markdown");
        assert_eq!(report.pages, 0);
        assert_eq!(report.figures_recovered, 0);
        assert_eq!(report.binder.question_count, 2);
        assert_eq!(report.binder.certification, "AI-900");
        assert_eq!(report.binder.needs_review_count, 0);

        let session = db::session_questions(
            &connection,
            report.binder.id,
            SessionMode::Practice,
            None,
            &db::RuleSet::default(),
        )
        .expect("session");
        assert_eq!(session.len(), 2);
        let multi = session.iter().find(|q| q.number == 2).expect("question 2");
        assert_eq!(multi.answer_letters, vec!['A', 'C']);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// A README that is not a question bank must say so rather than produce an empty binder.
    #[test]
    fn a_text_file_without_questions_is_refused() {
        let dir = std::env::temp_dir().join("oet-bank-empty");
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("readme.md");
        std::fs::write(&path, "# Notes\n\nNothing to import here.").expect("write");

        let mut connection = db::open_in_memory().expect("schema");
        let error = import_into(&mut connection, &[], &dir, &path, None).expect_err("refused");

        assert!(error.to_string().contains("no questions found"), "{error}");
        assert!(db::list_binders(&connection).expect("binders").is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_figure_is_served_as_a_data_url_and_only_a_content_hash_is_accepted() {
        let data_dir = std::env::temp_dir().join("oet-figure-read");
        let hash = "a".repeat(64);
        std::fs::create_dir_all(data_dir.join("figures")).expect("store");
        std::fs::write(figure_path(&data_dir, &hash), b"abc").expect("figure");

        assert_eq!(
            read_figure(&data_dir, &hash).expect("read"),
            "data:image/png;base64,YWJj"
        );

        // The hash arrives from the webview, so a path is not a name.
        for hostile in ["../../library.sqlite3", "..", "", &"a".repeat(63), "zz"] {
            let error = read_figure(&data_dir, hostile).expect_err(hostile);
            assert!(error.to_string().contains("not a content hash"), "{error}");
        }

        std::fs::remove_dir_all(&data_dir).ok();
    }

    /// The desktop path end to end: a real dump PDF in, a stored binder out. Skips loudly when the
    /// gitignored fixture is absent — see .agents/docs/03-ingest-pipeline.md §9.
    #[test]
    fn a_real_pdf_imports_into_a_stored_binder() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../crates/ingest/tests/fixtures/certleader-ai900.pdf");
        if !fixture.exists() {
            eprintln!("SKIPPED: certleader-ai900.pdf is missing");
            return;
        }

        let data_dir = std::env::temp_dir().join("oet-import-test");
        let _ = std::fs::remove_dir_all(&data_dir);
        let mut connection = db::open_in_memory().expect("schema");
        let report = import_into(
            &mut connection,
            &[pdf::vendored_library_dir()],
            &data_dir,
            &fixture,
            None,
        )
        .expect("import");

        assert_eq!(report.profile, "certleader");
        assert_eq!(report.pages, 7);
        assert_eq!(report.stub_markers, vec![15]);
        assert_eq!(report.binder.certification, "AI-900");
        assert_eq!(report.binder.question_count, 11);

        // The five drag-and-drop questions get their answer area back, so none is left unanswerable
        // and every figure is on disk under its own hash.
        assert_eq!(report.figures_recovered, 5);
        assert_eq!(report.binder.needs_source_count, 0);
        for question in db::list_questions(&connection, report.binder.id, false).expect("questions")
        {
            for hash in &question.figures {
                assert!(figure_path(&data_dir, hash).exists(), "{hash} is on disk");
            }
        }

        // What is still flagged for review is the real defect this dump has: it numbers two
        // consecutive questions 10.
        assert_eq!(report.binder.needs_review_count, 1);

        let scored = db::session_questions(
            &connection,
            report.binder.id,
            SessionMode::Practice,
            None,
            &db::RuleSet::default(),
        )
        .expect("session");
        assert_eq!(scored.len(), 11, "every question is drillable now");
        assert!(scored.iter().any(|q| !q.figures.is_empty()));

        std::fs::remove_dir_all(&data_dir).ok();
    }
}

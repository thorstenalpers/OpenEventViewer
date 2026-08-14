use std::path::Path;

use rs_fsrs::Card;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::dto::{
    Binder, Certification, ExamTimeline, Link, Note, QuestionDto, SessionMode, Template, Video,
};
use crate::error::AppResult;
use crate::srs;

pub const REVIEW_THRESHOLD: f64 = 0.75;

const SCHEMA: &str = r#"
-- A project *is* a binder: one certification, one imported file, and the questions, links,
-- videos and notes that hang off it. The UI calls it a project; the storage kept the older
-- name rather than rewriting every reference to it.
CREATE TABLE IF NOT EXISTS binders (
    id            INTEGER PRIMARY KEY,
    title         TEXT NOT NULL,
    certification TEXT NOT NULL,
    source_file   TEXT NOT NULL,
    profile       TEXT NOT NULL,
    imported_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS questions (
    id           INTEGER PRIMARY KEY,
    binder_id    INTEGER NOT NULL REFERENCES binders(id) ON DELETE CASCADE,
    number       INTEGER NOT NULL,
    confidence   REAL NOT NULL,
    needs_source INTEGER NOT NULL,
    payload      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS questions_binder ON questions(binder_id);

CREATE TABLE IF NOT EXISTS sessions (
    id                 INTEGER PRIMARY KEY,
    binder_id          INTEGER NOT NULL REFERENCES binders(id) ON DELETE CASCADE,
    mode               TEXT NOT NULL,
    seed               INTEGER,
    question_count     INTEGER,
    time_limit_seconds INTEGER,
    started_at         TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at        TEXT
);

CREATE TABLE IF NOT EXISTS attempts (
    id          INTEGER PRIMARY KEY,
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    given       TEXT NOT NULL,
    correct     INTEGER NOT NULL,
    elapsed_ms  INTEGER NOT NULL,
    at          TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS attempts_session ON attempts(session_id);
CREATE INDEX IF NOT EXISTS attempts_question ON attempts(question_id);

CREATE TABLE IF NOT EXISTS links (
    id          INTEGER PRIMARY KEY,
    binder_id   INTEGER NOT NULL REFERENCES binders(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id) ON DELETE CASCADE,
    url         TEXT NOT NULL,
    title       TEXT NOT NULL,
    UNIQUE(binder_id, url)
);
CREATE INDEX IF NOT EXISTS links_binder ON links(binder_id);

CREATE TABLE IF NOT EXISTS videos (
    id            INTEGER PRIMARY KEY,
    binder_id     INTEGER NOT NULL REFERENCES binders(id) ON DELETE CASCADE,
    question_id   INTEGER REFERENCES questions(id) ON DELETE CASCADE,
    url           TEXT NOT NULL,
    title         TEXT NOT NULL,
    start_seconds INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS videos_binder ON videos(binder_id);

CREATE TABLE IF NOT EXISTS notes (
    id          INTEGER PRIMARY KEY,
    binder_id   INTEGER NOT NULL REFERENCES binders(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id) ON DELETE CASCADE,
    body_md     TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS notes_binder ON notes(binder_id);

-- An exam as it exists before anyone studies for it: the code people call it by and the page the
-- vendor documents it on. Both together are the identity — the same code under a different study
-- guide is a different exam, and the same guide under two names is one exam typed twice.
CREATE TABLE IF NOT EXISTS templates (
    id      INTEGER PRIMARY KEY,
    name    TEXT NOT NULL,
    doc_url TEXT NOT NULL,
    UNIQUE(name, doc_url)
);

-- When an exam was passed, and how often. Several rows per project on purpose: a certification
-- expires and is taken again, and the history of that is the interesting part.
CREATE TABLE IF NOT EXISTS certifications (
    id        INTEGER PRIMARY KEY,
    binder_id INTEGER NOT NULL REFERENCES binders(id) ON DELETE CASCADE,
    passed_at TEXT NOT NULL,
    note      TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS certifications_binder ON certifications(binder_id);

-- Which steps of the study checklist are done. A row is the tick; no row is an open step, which
-- is why nothing here records the absence of one.
CREATE TABLE IF NOT EXISTS progress (
    binder_id INTEGER NOT NULL REFERENCES binders(id) ON DELETE CASCADE,
    step      TEXT NOT NULL,
    done_at   TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (binder_id, step)
);
"#;

/// One FSRS card per question: stability in days, difficulty on 1..10, and the timestamps the
/// forgetting curve needs to turn the two into a retrievability at review time.
///
/// Apart from `SCHEMA` because the migration off SM-2 drops the table and builds it again.
const SCHEDULING_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS scheduling (
    question_id    INTEGER PRIMARY KEY REFERENCES questions(id) ON DELETE CASCADE,
    due_at         TEXT NOT NULL,
    last_review_at TEXT NOT NULL,
    stability      REAL NOT NULL,
    difficulty     REAL NOT NULL,
    elapsed_days   INTEGER NOT NULL,
    scheduled_days INTEGER NOT NULL,
    reps           INTEGER NOT NULL,
    lapses         INTEGER NOT NULL,
    state          INTEGER NOT NULL
);
"#;

pub fn open(path: &Path) -> AppResult<Connection> {
    prepare(Connection::open(path)?)
}

fn prepare(mut connection: Connection) -> AppResult<Connection> {
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(SCHEMA)?;
    connection.execute_batch(SCHEDULING_SCHEMA)?;
    migrate_scheduling(&mut connection)?;
    add_columns(&connection)?;
    seed_templates(&connection)?;
    Ok(connection)
}

/// Columns added to tables that already existed in someone's database.
///
/// `CREATE TABLE IF NOT EXISTS` covers a new table and does nothing for a new column, so each one
/// is checked against `pragma_table_info` and added when it is missing. Every one of them carries a
/// default, because the rows already there have no value to offer.
fn add_columns(connection: &Connection) -> AppResult<()> {
    const COLUMNS: &[(&str, &str, &str)] = &[
        // The exam's documentation page, taken from the template it was created from.
        ("binders", "doc_url", "TEXT NOT NULL DEFAULT ''"),
        // The catalog entry this binder was published as, so republishing replaces it rather than
        // adding a second copy. Null until it is published, and null again after a withdrawal.
        ("binders", "remote_id", "TEXT"),
        ("links", "description", "TEXT NOT NULL DEFAULT ''"),
        // What kind of thing is at the other end: course, video, docs, other.
        ("links", "kind", "TEXT NOT NULL DEFAULT 'other'"),
        ("links", "minutes", "INTEGER"),
        // A video that lives on this machine rather than on a site.
        ("videos", "is_local", "INTEGER NOT NULL DEFAULT 0"),
        ("videos", "minutes", "INTEGER"),
    ];

    for (table, column, definition) in COLUMNS {
        let present = connection
            .prepare(&format!(
                "SELECT 1 FROM pragma_table_info('{table}') WHERE name = ?1"
            ))?
            .exists([column])?;
        if !present {
            connection.execute_batch(&format!(
                "ALTER TABLE {table} ADD COLUMN {column} {definition};"
            ))?;
        }
    }
    Ok(())
}

/// The exams offered by name when a project is created, written once into an empty table.
///
/// Only when it is empty: a template the user deleted must stay deleted, and one they edited must
/// not be overwritten on the next start. Every URL was requested and answered 200 on 14 August
/// 2026; they all follow Microsoft's own `study-guides/<code>` pattern.
fn seed_templates(connection: &Connection) -> AppResult<()> {
    const CATALOGUE: &[&str] = &[
        "AZ-900", "AI-900", "DP-900", "MS-900", "SC-900", "PL-900", "GH-900", "AZ-104", "AZ-204",
        "AZ-305", "AZ-500", "AI-102", "DP-203", "SC-200", "SC-300",
    ];

    let known: i64 =
        connection.query_row("SELECT COUNT(*) FROM templates", [], |row| row.get(0))?;
    if known > 0 {
        return Ok(());
    }

    for name in CATALOGUE {
        connection.execute(
            "INSERT OR IGNORE INTO templates (name, doc_url) VALUES (?1, ?2)",
            rusqlite::params![
                name,
                format!(
                    "https://learn.microsoft.com/en-us/credentials/certifications/resources/study-guides/{}",
                    name.to_lowercase()
                )
            ],
        )?;
    }
    Ok(())
}

/// SM-2's `ease` and FSRS's `difficulty` are different quantities on different scales, and
/// `interval_days` is an output of the old model rather than a memory state — nothing in the old
/// table converts. `attempts` is append-only and holds every review this library has ever seen,
/// so the state is not converted, it is replayed from the log that produced it.
///
/// In one transaction because the old columns are what marks the database as unmigrated: a replay
/// that stopped half way would leave a table nothing recognises as needing finishing, and the
/// questions past the break would silently never get their schedule back.
fn migrate_scheduling(connection: &mut Connection) -> AppResult<()> {
    let legacy = connection
        .prepare("SELECT 1 FROM pragma_table_info('scheduling') WHERE name = 'ease'")?
        .exists([])?;
    if !legacy {
        return Ok(());
    }

    let transaction = connection.transaction()?;
    transaction.execute_batch("DROP TABLE scheduling;")?;
    transaction.execute_batch(SCHEDULING_SCHEMA)?;

    let reviews = {
        let mut statement =
            transaction.prepare("SELECT question_id, correct, at FROM attempts ORDER BY at, id")?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)? != 0,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };

    for (question_id, correct, at) in reviews {
        reschedule_at(&transaction, question_id, correct, &at)?;
    }
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    prepare(Connection::open_in_memory()?)
}

/// SQLite owns the clock here. Every other timestamp in the schema comes from `datetime('now')`,
/// and a second source would drift against them.
pub fn now(connection: &Connection) -> AppResult<String> {
    Ok(connection.query_row("SELECT datetime('now')", [], |row| row.get(0))?)
}

const BINDER_COLUMNS: &str = r#"
    b.id, b.title, b.certification, b.doc_url, b.source_file, b.profile, b.imported_at,
    (SELECT COUNT(*) FROM questions q WHERE q.binder_id = b.id),
    (SELECT COUNT(*) FROM questions q WHERE q.binder_id = b.id AND q.confidence < ?1),
    (SELECT COUNT(*) FROM questions q WHERE q.binder_id = b.id AND q.needs_source = 1),
    (SELECT MAX(a.at) FROM attempts a JOIN questions q ON q.id = a.question_id WHERE q.binder_id = b.id),
    (SELECT COUNT(*) FROM attempts a JOIN questions q ON q.id = a.question_id WHERE q.binder_id = b.id),
    (SELECT AVG(a.correct) FROM attempts a JOIN questions q ON q.id = a.question_id WHERE q.binder_id = b.id)
"#;

fn binder_from_row(row: &Row<'_>) -> rusqlite::Result<Binder> {
    Ok(Binder {
        id: row.get(0)?,
        title: row.get(1)?,
        certification: row.get(2)?,
        doc_url: row.get(3)?,
        source_file: row.get(4)?,
        profile: row.get(5)?,
        imported_at: row.get(6)?,
        question_count: row.get(7)?,
        needs_review_count: row.get(8)?,
        needs_source_count: row.get(9)?,
        last_studied_at: row.get(10)?,
        attempt_count: row.get(11)?,
        accuracy: row.get(12)?,
    })
}

pub fn list_binders(connection: &Connection) -> AppResult<Vec<Binder>> {
    let sql = format!("SELECT {BINDER_COLUMNS} FROM binders b ORDER BY b.title");
    let mut statement = connection.prepare(&sql)?;
    let binders = statement
        .query_map(params![REVIEW_THRESHOLD], binder_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(binders)
}

pub fn binder(connection: &Connection, binder_id: i64) -> AppResult<Option<Binder>> {
    let sql = format!("SELECT {BINDER_COLUMNS} FROM binders b WHERE b.id = ?2");
    let found = connection
        .query_row(&sql, params![REVIEW_THRESHOLD, binder_id], binder_from_row)
        .optional()?;
    Ok(found)
}

/// Writes a binder and its questions, and returns the new question ids in the order given so a
/// caller can attach links, videos and notes without a second lookup.
pub fn insert_binder(
    connection: &mut Connection,
    title: &str,
    certification: &str,
    source_file: &str,
    profile: &str,
    questions: &[QuestionDto],
) -> AppResult<(i64, Vec<i64>)> {
    let transaction = connection.transaction()?;
    transaction.execute(
        "INSERT INTO binders (title, certification, source_file, profile) VALUES (?1, ?2, ?3, ?4)",
        params![title, certification, source_file, profile],
    )?;
    let binder_id = transaction.last_insert_rowid();
    let question_ids = write_questions(&transaction, binder_id, questions)?;
    transaction.commit()?;
    Ok((binder_id, question_ids))
}

/// Creates a project with nothing in it yet, to be filled by a later import.
///
/// A project is a binder, so this is the same row an import would have made — only without the
/// questions. That keeps one table rather than a project that owns a binder that owns everything.
pub fn create_project(
    connection: &mut Connection,
    title: &str,
    certification: &str,
    doc_url: &str,
) -> AppResult<i64> {
    connection.execute(
        "INSERT INTO binders (title, certification, source_file, profile, doc_url)
         VALUES (?1, ?2, '', '', ?3)",
        params![title, certification, doc_url],
    )?;
    Ok(connection.last_insert_rowid())
}

/// Puts an import into an existing, still-empty project.
///
/// Refuses a project that already has questions: a project holds one file, and silently appending
/// a second import would merge two exams into one score.
pub fn fill_project(
    connection: &mut Connection,
    binder_id: i64,
    source_file: &str,
    profile: &str,
    questions: &[QuestionDto],
) -> AppResult<Vec<i64>> {
    let existing: i64 = connection.query_row(
        "SELECT COUNT(*) FROM questions WHERE binder_id = ?1",
        [binder_id],
        |row| row.get(0),
    )?;
    if existing > 0 {
        return Err(crate::error::AppError::Message(format!(
            "project {binder_id} already holds {existing} questions — a project takes one file"
        )));
    }

    let transaction = connection.transaction()?;
    transaction.execute(
        "UPDATE binders SET source_file = ?2, profile = ?3 WHERE id = ?1",
        params![binder_id, source_file, profile],
    )?;
    let question_ids = write_questions(&transaction, binder_id, questions)?;
    transaction.commit()?;
    Ok(question_ids)
}

fn write_questions(
    transaction: &rusqlite::Transaction<'_>,
    binder_id: i64,
    questions: &[QuestionDto],
) -> AppResult<Vec<i64>> {
    let mut question_ids = Vec::with_capacity(questions.len());
    let mut insert_question = transaction.prepare(
        "INSERT INTO questions (binder_id, number, confidence, needs_source, payload)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut insert_link = transaction.prepare(
        "INSERT OR IGNORE INTO links (binder_id, question_id, url, title) VALUES (?1, ?2, ?3, ?4)",
    )?;

    for question in questions {
        insert_question.execute(params![
            binder_id,
            question.number,
            question.confidence,
            question.needs_source as i64,
            serde_json::to_string(question)?,
        ])?;
        let question_id = transaction.last_insert_rowid();
        question_ids.push(question_id);

        // Every `Reference:` the extractor found becomes a project bookmark, so "read the doc
        // behind this question" needs no second pass over the source.
        for url in &question.references {
            insert_link.execute(params![binder_id, question_id, url, link_title(url)])?;
        }
    }

    Ok(question_ids)
}

/// A readable label for a bare URL: the last meaningful path segment, else the host.
fn link_title(url: &str) -> String {
    let without_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let mut segments = without_scheme.split('/').filter(|s| !s.is_empty());
    let host = segments.next().unwrap_or(without_scheme);
    let last = segments.rfind(|segment| !segment.contains('=') && segment.len() > 2);

    match last {
        Some(segment) => segment
            .trim_end_matches(".html")
            .trim_end_matches(".htm")
            .replace(['-', '_'], " "),
        None => host.trim_start_matches("www.").to_string(),
    }
}

pub fn delete_binder(connection: &Connection, binder_id: i64) -> AppResult<()> {
    connection.execute("DELETE FROM binders WHERE id = ?1", params![binder_id])?;
    Ok(())
}

fn question_from_row(row: &Row<'_>) -> rusqlite::Result<QuestionDto> {
    let id: i64 = row.get(0)?;
    let payload: String = row.get(1)?;
    let mut dto: QuestionDto = serde_json::from_str(&payload).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(error))
    })?;
    dto.id = id;
    Ok(dto)
}

pub fn list_questions(
    connection: &Connection,
    binder_id: i64,
    only_review: bool,
) -> AppResult<Vec<QuestionDto>> {
    let sql = if only_review {
        "SELECT id, payload FROM questions WHERE binder_id = ?1 AND confidence < ?2 ORDER BY number"
    } else {
        "SELECT id, payload FROM questions WHERE binder_id = ?1 AND ?2 IS NOT NULL ORDER BY number"
    };
    let mut statement = connection.prepare(sql)?;
    let questions = statement
        .query_map(params![binder_id, REVIEW_THRESHOLD], question_from_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(questions)
}

/// Picks the question set for a session.
///
/// Focus reads the wrong answers of one finished session out of `attempts` rather than a stored
/// list, so the set cannot drift from what actually happened.
pub fn session_questions(
    connection: &Connection,
    binder_id: i64,
    mode: SessionMode,
    source_session_id: Option<i64>,
    rules: &RuleSet,
) -> AppResult<Vec<QuestionDto>> {
    if matches!(mode, SessionMode::Exam | SessionMode::Challenge) {
        // Ordered by number first, then shuffled by the seed: the database's own row order must
        // never leak into a challenge, or two machines get different exams from the same seed.
        let mut statement = connection.prepare(
            "SELECT id, payload FROM questions
             WHERE binder_id = ?1 AND needs_source = 0
             ORDER BY number, id",
        )?;
        let mut questions = statement
            .query_map(params![binder_id], question_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        shuffle(&mut questions, rules.seed.unwrap_or(1));
        if let Some(count) = rules.question_count {
            questions.truncate(count.max(0) as usize);
        }
        return Ok(questions);
    }

    let questions = match (mode, source_session_id) {
        (SessionMode::Focus, Some(session_id)) => {
            let mut statement = connection.prepare(
                "SELECT q.id, q.payload
                 FROM questions q
                 JOIN attempts a ON a.question_id = q.id
                 WHERE a.session_id = ?1 AND a.correct = 0 AND q.needs_source = 0
                 GROUP BY q.id
                 ORDER BY q.number",
            )?;
            let rows = statement
                .query_map(params![session_id], question_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        }
        // The persistent weak pool. A question enters it by being missed once and leaves it only
        // after two correct answers in a row *in different sessions* — twice in the same sitting is
        // short-term memory, not learning, and a single lucky guess should not clear anything.
        (SessionMode::Weak, _) => {
            let mut statement = connection.prepare(
                "SELECT q.id, q.payload
                 FROM questions q
                 WHERE q.binder_id = ?1
                   AND q.needs_source = 0
                   AND EXISTS (SELECT 1 FROM attempts a
                               WHERE a.question_id = q.id AND a.correct = 0)
                   AND NOT (
                     (SELECT COUNT(*) FROM (
                        SELECT a.correct FROM attempts a
                        WHERE a.question_id = q.id ORDER BY a.id DESC LIMIT 2
                      ) recent WHERE recent.correct = 1) = 2
                     AND
                     (SELECT COUNT(DISTINCT recent.session_id) FROM (
                        SELECT a.session_id FROM attempts a
                        WHERE a.question_id = q.id ORDER BY a.id DESC LIMIT 2
                      ) recent) = 2
                   )
                 ORDER BY q.number",
            )?;
            let rows = statement
                .query_map(params![binder_id], question_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        }
        (SessionMode::Due, _) => {
            let mut statement = connection.prepare(
                "SELECT q.id, q.payload
                 FROM questions q
                 LEFT JOIN scheduling s ON s.question_id = q.id
                 WHERE q.binder_id = ?1 AND q.needs_source = 0
                   AND (s.due_at IS NULL OR s.due_at <= datetime('now'))
                 ORDER BY RANDOM()",
            )?;
            let rows = statement
                .query_map(params![binder_id], question_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        }
        _ => {
            let mut statement = connection.prepare(
                "SELECT id, payload FROM questions
                 WHERE binder_id = ?1 AND needs_source = 0
                 ORDER BY RANDOM()",
            )?;
            let rows = statement
                .query_map(params![binder_id], question_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        }
    };
    Ok(questions)
}

pub fn create_session(
    connection: &Connection,
    binder_id: i64,
    mode: SessionMode,
    rules: &RuleSet,
) -> AppResult<i64> {
    connection.execute(
        "INSERT INTO sessions (binder_id, mode, seed, question_count, time_limit_seconds)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            binder_id,
            mode.as_str(),
            rules.seed,
            rules.question_count,
            rules.time_limit_seconds
        ],
    )?;
    Ok(connection.last_insert_rowid())
}

/// What makes two runs of a challenge comparable: the same questions, in the same order, under the
/// same clock. `RANDOM()` in SQL cannot be seeded, so a seeded order is drawn in Rust instead.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSet {
    pub seed: Option<i64>,
    pub question_count: Option<i64>,
    pub time_limit_seconds: Option<i64>,
}

/// xorshift64*: small, deterministic, and identical on every machine — which is the whole point of
/// a shared seed.
pub fn shuffle<T>(items: &mut [T], seed: i64) {
    let mut state = (seed as u64) | 1;
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    for index in (1..items.len()).rev() {
        let pick = (next() % (index as u64 + 1)) as usize;
        items.swap(index, pick);
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeResult {
    pub session_id: i64,
    pub seed: i64,
    pub finished_at: String,
    pub total: i64,
    pub correct: i64,
    pub elapsed_ms: i64,
}

/// The leaderboard for one seed: same questions, same order, so accuracy and time compare.
pub fn challenge_results(
    connection: &Connection,
    binder_id: i64,
    seed: i64,
) -> AppResult<Vec<ChallengeResult>> {
    let mut statement = connection.prepare(
        "SELECT s.id, s.seed, s.finished_at,
                COUNT(a.id), COALESCE(SUM(a.correct), 0), COALESCE(SUM(a.elapsed_ms), 0)
         FROM sessions s
         JOIN attempts a ON a.session_id = s.id
         WHERE s.binder_id = ?1 AND s.seed = ?2 AND s.finished_at IS NOT NULL
         GROUP BY s.id
         ORDER BY COALESCE(SUM(a.correct), 0) DESC, COALESCE(SUM(a.elapsed_ms), 0) ASC",
    )?;
    let results = statement
        .query_map(params![binder_id, seed], |row| {
            Ok(ChallengeResult {
                session_id: row.get(0)?,
                seed: row.get(1)?,
                finished_at: row.get(2)?,
                total: row.get(3)?,
                correct: row.get(4)?,
                elapsed_ms: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(results)
}

/// A finished run, read back without finishing it again, for posting to a catalog board.
///
/// A session with no seed is refused rather than posted under seed 0: the board is per seed, and an
/// unseeded run shares its questions with nothing.
pub fn session_result(connection: &Connection, session_id: i64) -> AppResult<ChallengeResult> {
    let (seed, finished_at): (Option<i64>, Option<String>) = connection.query_row(
        "SELECT seed, finished_at FROM sessions WHERE id = ?1",
        params![session_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let Some(seed) = seed else {
        return Err(crate::error::AppError::Message(format!(
            "session {session_id} ran without a seed, so there is no board to post it to"
        )));
    };
    let Some(finished_at) = finished_at else {
        return Err(crate::error::AppError::Message(format!(
            "session {session_id} has not finished"
        )));
    };

    let (total, correct, elapsed_ms): (i64, i64, i64) = connection.query_row(
        "SELECT COUNT(*), COALESCE(SUM(correct), 0), COALESCE(SUM(elapsed_ms), 0)
         FROM attempts WHERE session_id = ?1",
        params![session_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    Ok(ChallengeResult {
        session_id,
        seed,
        finished_at,
        total,
        correct,
        elapsed_ms,
    })
}

pub fn insert_attempt(
    connection: &Connection,
    session_id: i64,
    question_id: i64,
    given: &str,
    correct: bool,
    elapsed_ms: i64,
) -> AppResult<()> {
    connection.execute(
        "INSERT INTO attempts (session_id, question_id, given, correct, elapsed_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![session_id, question_id, given, correct as i64, elapsed_ms],
    )?;
    Ok(())
}

/// One attempt, one FSRS review. The interval is not multiplied up from the last one — it falls
/// out of the stability the model now attributes to this question, so a question answered right
/// after three months moves further than the same question answered right the same afternoon.
pub fn reschedule(connection: &Connection, question_id: i64, correct: bool) -> AppResult<()> {
    reschedule_at(connection, question_id, correct, &now(connection)?)
}

fn reschedule_at(
    connection: &Connection,
    question_id: i64,
    correct: bool,
    at: &str,
) -> AppResult<()> {
    let reviewed = srs::review(
        load_card(connection, question_id)?,
        srs::parse(at)?,
        correct,
    );
    store_card(connection, question_id, &reviewed)
}

/// A question with no row has never been reviewed, and that is exactly what a default `Card` is.
fn load_card(connection: &Connection, question_id: i64) -> AppResult<Card> {
    let stored = connection
        .query_row(
            "SELECT due_at, last_review_at, stability, difficulty, elapsed_days, scheduled_days,
                    reps, lapses, state
             FROM scheduling WHERE question_id = ?1",
            params![question_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i32>(6)?,
                    row.get::<_, i32>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .optional()?;

    let Some((
        due,
        last_review,
        stability,
        difficulty,
        elapsed_days,
        scheduled_days,
        reps,
        lapses,
        state,
    )) = stored
    else {
        return Ok(Card::default());
    };

    Ok(Card {
        due: srs::parse(&due)?,
        last_review: srs::parse(&last_review)?,
        stability,
        difficulty,
        elapsed_days,
        scheduled_days,
        reps,
        lapses,
        state: srs::state_from(state),
    })
}

fn store_card(connection: &Connection, question_id: i64, card: &Card) -> AppResult<()> {
    connection.execute(
        "INSERT INTO scheduling (question_id, due_at, last_review_at, stability, difficulty,
                                 elapsed_days, scheduled_days, reps, lapses, state)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(question_id) DO UPDATE SET
            due_at = excluded.due_at,
            last_review_at = excluded.last_review_at,
            stability = excluded.stability,
            difficulty = excluded.difficulty,
            elapsed_days = excluded.elapsed_days,
            scheduled_days = excluded.scheduled_days,
            reps = excluded.reps,
            lapses = excluded.lapses,
            state = excluded.state",
        params![
            question_id,
            srs::stamp(card.due),
            srs::stamp(card.last_review),
            card.stability,
            card.difficulty,
            card.elapsed_days,
            card.scheduled_days,
            card.reps,
            card.lapses,
            srs::state_code(card.state),
        ],
    )?;
    Ok(())
}

/// One question's whole FSRS card as it is stored.
///
/// The card is the unit that crosses a sync boundary. Carrying only stability and a due date would
/// mean inventing `reps`, `state` and the two day counts on arrival, and the scheduler reads all
/// four when it works out the next interval.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scheduling {
    pub due_at: String,
    pub last_review_at: String,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub reps: i64,
    pub lapses: i64,
    pub state: i64,
}

pub fn scheduling(connection: &Connection, question_id: i64) -> AppResult<Option<Scheduling>> {
    let found = connection
        .query_row(
            "SELECT due_at, last_review_at, stability, difficulty, elapsed_days, scheduled_days,
                    reps, lapses, state
             FROM scheduling WHERE question_id = ?1",
            params![question_id],
            |row| {
                Ok(Scheduling {
                    due_at: row.get(0)?,
                    last_review_at: row.get(1)?,
                    stability: row.get(2)?,
                    difficulty: row.get(3)?,
                    elapsed_days: row.get(4)?,
                    scheduled_days: row.get(5)?,
                    reps: row.get(6)?,
                    lapses: row.get(7)?,
                    state: row.get(8)?,
                })
            },
        )
        .optional()?;
    Ok(found)
}

pub fn put_scheduling(
    connection: &Connection,
    question_id: i64,
    card: &Scheduling,
) -> AppResult<()> {
    connection.execute(
        "INSERT INTO scheduling (question_id, due_at, last_review_at, stability, difficulty,
                                 elapsed_days, scheduled_days, reps, lapses, state)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(question_id) DO UPDATE SET
            due_at = excluded.due_at,
            last_review_at = excluded.last_review_at,
            stability = excluded.stability,
            difficulty = excluded.difficulty,
            elapsed_days = excluded.elapsed_days,
            scheduled_days = excluded.scheduled_days,
            reps = excluded.reps,
            lapses = excluded.lapses,
            state = excluded.state",
        params![
            question_id,
            card.due_at,
            card.last_review_at,
            card.stability,
            card.difficulty,
            card.elapsed_days,
            card.scheduled_days,
            card.reps,
            card.lapses,
            card.state,
        ],
    )?;
    Ok(())
}

/// Every question in the library with how often it has been answered, for the progress sync.
///
/// Across binders on purpose: the same question imported twice is one question to sync, and which
/// binder a copy sits in is a local arrangement the other machine does not share.
pub fn questions_with_attempts(connection: &Connection) -> AppResult<Vec<(QuestionDto, i64, i64)>> {
    let mut statement = connection.prepare(
        "SELECT q.id, q.payload,
                (SELECT COUNT(*) FROM attempts a WHERE a.question_id = q.id),
                (SELECT COALESCE(SUM(a.correct), 0) FROM attempts a WHERE a.question_id = q.id)
         FROM questions q
         ORDER BY q.id",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok((question_from_row(row)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn remote_id(connection: &Connection, binder_id: i64) -> AppResult<Option<String>> {
    let found = connection
        .query_row(
            "SELECT remote_id FROM binders WHERE id = ?1",
            params![binder_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten();
    Ok(found)
}

pub fn set_remote_id(
    connection: &Connection,
    binder_id: i64,
    remote_id: Option<&str>,
) -> AppResult<()> {
    connection.execute(
        "UPDATE binders SET remote_id = ?2 WHERE id = ?1",
        params![binder_id, remote_id],
    )?;
    Ok(())
}

/// The local binders that are published, so the catalog can mark its own entries without the UI
/// pairing two lists by title.
pub fn published_binders(connection: &Connection) -> AppResult<Vec<(String, i64)>> {
    let mut statement = connection
        .prepare("SELECT remote_id, id FROM binders WHERE remote_id IS NOT NULL ORDER BY id")?;
    let rows = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn insert_link(
    connection: &Connection,
    binder_id: i64,
    question_id: Option<i64>,
    url: &str,
    title: &str,
) -> AppResult<()> {
    connection.execute(
        "INSERT OR IGNORE INTO links (binder_id, question_id, url, title) VALUES (?1, ?2, ?3, ?4)",
        params![binder_id, question_id, url, title],
    )?;
    Ok(())
}

pub fn list_links(connection: &Connection, binder_id: i64) -> AppResult<Vec<Link>> {
    let mut statement = connection.prepare(
        "SELECT id, question_id, url, title, description, kind, minutes FROM links
         WHERE binder_id = ?1 ORDER BY title",
    )?;
    let links = statement
        .query_map(params![binder_id], |row| {
            Ok(Link {
                id: row.get(0)?,
                question_id: row.get(1)?,
                url: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                kind: row.get(5)?,
                minutes: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(links)
}

/// Adds a study link, or updates the one already stored under that address.
///
/// Upsert rather than insert: the table is unique on (binder, url), and pasting a link twice is
/// how someone corrects its title, not an error to report back at them.
pub fn save_link(connection: &Connection, binder_id: i64, link: &Link) -> AppResult<()> {
    connection.execute(
        "INSERT INTO links (binder_id, question_id, url, title, description, kind, minutes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(binder_id, url) DO UPDATE SET
            question_id = excluded.question_id,
            title       = excluded.title,
            description = excluded.description,
            kind        = excluded.kind,
            minutes     = excluded.minutes",
        params![
            binder_id,
            link.question_id,
            link.url,
            link.title,
            link.description,
            link.kind,
            link.minutes
        ],
    )?;
    Ok(())
}

pub fn delete_link(connection: &Connection, binder_id: i64, link_id: i64) -> AppResult<()> {
    connection.execute(
        "DELETE FROM links WHERE id = ?1 AND binder_id = ?2",
        params![link_id, binder_id],
    )?;
    Ok(())
}

pub fn list_templates(connection: &Connection) -> AppResult<Vec<Template>> {
    let mut statement =
        connection.prepare("SELECT id, name, doc_url FROM templates ORDER BY name")?;
    let templates = statement
        .query_map([], |row| {
            Ok(Template {
                id: row.get(0)?,
                name: row.get(1)?,
                doc_url: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(templates)
}

/// Stores a template, or returns the one already there.
///
/// Name and address together are the identity, so adding the same pair twice is not an error — it
/// is the same exam, and the caller gets the row that already describes it.
pub fn save_template(connection: &Connection, name: &str, doc_url: &str) -> AppResult<i64> {
    connection.execute(
        "INSERT OR IGNORE INTO templates (name, doc_url) VALUES (?1, ?2)",
        params![name, doc_url],
    )?;
    Ok(connection.query_row(
        "SELECT id FROM templates WHERE name = ?1 AND doc_url = ?2",
        params![name, doc_url],
        |row| row.get(0),
    )?)
}

pub fn delete_template(connection: &Connection, template_id: i64) -> AppResult<()> {
    connection.execute("DELETE FROM templates WHERE id = ?1", [template_id])?;
    Ok(())
}

pub fn list_certifications(
    connection: &Connection,
    binder_id: i64,
) -> AppResult<Vec<Certification>> {
    let mut statement = connection.prepare(
        "SELECT id, passed_at, note FROM certifications WHERE binder_id = ?1 ORDER BY passed_at",
    )?;
    let dates = statement
        .query_map([binder_id], |row| {
            Ok(Certification {
                id: row.get(0)?,
                passed_at: row.get(1)?,
                note: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(dates)
}

pub fn add_certification(
    connection: &Connection,
    binder_id: i64,
    passed_at: &str,
    note: &str,
) -> AppResult<i64> {
    connection.execute(
        "INSERT INTO certifications (binder_id, passed_at, note) VALUES (?1, ?2, ?3)",
        params![binder_id, passed_at, note],
    )?;
    Ok(connection.last_insert_rowid())
}

pub fn delete_certification(
    connection: &Connection,
    binder_id: i64,
    certification_id: i64,
) -> AppResult<()> {
    connection.execute(
        "DELETE FROM certifications WHERE id = ?1 AND binder_id = ?2",
        params![certification_id, binder_id],
    )?;
    Ok(())
}

/// The steps ticked off for one project. A step nobody has ticked has no row, so absence is the
/// open state and there is nothing to store for it.
pub fn list_progress(connection: &Connection, binder_id: i64) -> AppResult<Vec<String>> {
    let mut statement =
        connection.prepare("SELECT step FROM progress WHERE binder_id = ?1 ORDER BY done_at")?;
    let steps = statement
        .query_map([binder_id], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(steps)
}

pub fn set_progress(
    connection: &Connection,
    binder_id: i64,
    step: &str,
    done: bool,
) -> AppResult<()> {
    if done {
        connection.execute(
            "INSERT OR IGNORE INTO progress (binder_id, step) VALUES (?1, ?2)",
            params![binder_id, step],
        )?;
    } else {
        connection.execute(
            "DELETE FROM progress WHERE binder_id = ?1 AND step = ?2",
            params![binder_id, step],
        )?;
    }
    Ok(())
}

/// Every exam on one time axis: when it was created, when it was last studied, and each date it
/// was passed.
///
/// One query per exam for the dates rather than a join: an exam has a handful of them at most, and
/// the alternative is grouping rows back together in Rust for no gain.
pub fn timeline(connection: &Connection) -> AppResult<Vec<ExamTimeline>> {
    let mut statement = connection.prepare(
        "SELECT b.id, b.title, b.certification, b.imported_at,
                (SELECT MAX(a.at) FROM attempts a
                   JOIN questions q ON q.id = a.question_id WHERE q.binder_id = b.id),
                (SELECT COUNT(*) FROM questions q WHERE q.binder_id = b.id)
         FROM binders b ORDER BY b.imported_at",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok(ExamTimeline {
                binder_id: row.get(0)?,
                title: row.get(1)?,
                certification: row.get(2)?,
                started_at: row.get(3)?,
                last_studied_at: row.get(4)?,
                question_count: row.get(5)?,
                passed: Vec::new(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    rows.into_iter()
        .map(|mut exam| {
            exam.passed = list_certifications(connection, exam.binder_id)?
                .into_iter()
                .map(|entry| entry.passed_at)
                .collect();
            Ok(exam)
        })
        .collect()
}

pub fn insert_video(connection: &Connection, binder_id: i64, video: &Video) -> AppResult<i64> {
    connection.execute(
        "INSERT INTO videos (binder_id, question_id, url, title, start_seconds)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            binder_id,
            video.question_id,
            video.url,
            video.title,
            video.start_seconds
        ],
    )?;
    Ok(connection.last_insert_rowid())
}

pub fn list_videos(connection: &Connection, binder_id: i64) -> AppResult<Vec<Video>> {
    let mut statement = connection.prepare(
        "SELECT id, question_id, url, title, start_seconds FROM videos
         WHERE binder_id = ?1 ORDER BY title",
    )?;
    let videos = statement
        .query_map(params![binder_id], |row| {
            Ok(Video {
                id: row.get(0)?,
                question_id: row.get(1)?,
                url: row.get(2)?,
                title: row.get(3)?,
                start_seconds: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(videos)
}

pub fn delete_video(connection: &Connection, video_id: i64) -> AppResult<()> {
    connection.execute("DELETE FROM videos WHERE id = ?1", params![video_id])?;
    Ok(())
}

pub fn upsert_note(connection: &Connection, binder_id: i64, note: &Note) -> AppResult<i64> {
    if note.id > 0 {
        connection.execute(
            "UPDATE notes SET body_md = ?2, updated_at = datetime('now') WHERE id = ?1",
            params![note.id, note.body_md],
        )?;
        return Ok(note.id);
    }
    connection.execute(
        "INSERT INTO notes (binder_id, question_id, body_md) VALUES (?1, ?2, ?3)",
        params![binder_id, note.question_id, note.body_md],
    )?;
    Ok(connection.last_insert_rowid())
}

pub fn list_notes(connection: &Connection, binder_id: i64) -> AppResult<Vec<Note>> {
    let mut statement = connection.prepare(
        "SELECT id, question_id, body_md, updated_at FROM notes
         WHERE binder_id = ?1 ORDER BY updated_at DESC",
    )?;
    let notes = statement
        .query_map(params![binder_id], |row| {
            Ok(Note {
                id: row.get(0)?,
                question_id: row.get(1)?,
                body_md: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(notes)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionStat {
    pub question_id: i64,
    pub number: u32,
    pub topic: Option<u16>,
    pub stem: String,
    pub attempts: i64,
    pub correct: i64,
    /// `None` until the question has been answered once — an unanswered question is not 0 %.
    pub accuracy: Option<f64>,
    pub average_ms: Option<f64>,
    pub lapses: i64,
    pub due_at: Option<String>,
    pub needs_source: bool,
}

/// Everything the statistics view shows, derived from `attempts` and `scheduling` rather than kept
/// as running totals — a counter that drifts from the attempt log would be invisible.
pub fn question_stats(connection: &Connection, binder_id: i64) -> AppResult<Vec<QuestionStat>> {
    let mut statement = connection.prepare(
        "SELECT q.id, q.payload, q.needs_source,
                (SELECT COUNT(*)                FROM attempts a  WHERE a.question_id = q.id),
                (SELECT COALESCE(SUM(a.correct), 0) FROM attempts a WHERE a.question_id = q.id),
                (SELECT AVG(a.elapsed_ms)       FROM attempts a  WHERE a.question_id = q.id),
                (SELECT s.lapses                FROM scheduling s WHERE s.question_id = q.id),
                (SELECT s.due_at                FROM scheduling s WHERE s.question_id = q.id)
         FROM questions q
         WHERE q.binder_id = ?1
         ORDER BY q.number, q.id",
    )?;

    let stats = statement
        .query_map(params![binder_id], |row| {
            let payload: String = row.get(1)?;
            let question: QuestionDto = serde_json::from_str(&payload).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            let attempts: i64 = row.get(3)?;
            let correct: i64 = row.get(4)?;
            Ok(QuestionStat {
                question_id: row.get(0)?,
                number: question.number,
                topic: question.topic,
                stem: question.stem,
                attempts,
                correct,
                accuracy: (attempts > 0).then(|| correct as f64 / attempts as f64),
                average_ms: row.get(5)?,
                lapses: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                due_at: row.get(7)?,
                needs_source: row.get::<_, i64>(2)? != 0,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(stats)
}

pub struct SessionTotals {
    pub binder_id: i64,
    pub mode: SessionMode,
    pub total: i64,
    pub correct: i64,
    pub elapsed_ms: i64,
    pub wrong_question_ids: Vec<i64>,
}

pub fn finish_session(connection: &Connection, session_id: i64) -> AppResult<SessionTotals> {
    connection.execute(
        "UPDATE sessions SET finished_at = datetime('now') WHERE id = ?1",
        params![session_id],
    )?;

    let (binder_id, mode): (i64, String) = connection.query_row(
        "SELECT binder_id, mode FROM sessions WHERE id = ?1",
        params![session_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let (total, correct, elapsed_ms): (i64, i64, i64) = connection.query_row(
        "SELECT COUNT(*), COALESCE(SUM(correct), 0), COALESCE(SUM(elapsed_ms), 0)
         FROM attempts WHERE session_id = ?1",
        params![session_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let mut statement = connection.prepare(
        "SELECT DISTINCT question_id FROM attempts WHERE session_id = ?1 AND correct = 0",
    )?;
    let wrong_question_ids = statement
        .query_map(params![session_id], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<i64>>>()?;

    Ok(SessionTotals {
        binder_id,
        mode: SessionMode::from_str(&mode),
        total,
        correct,
        elapsed_ms,
        wrong_question_ids,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentSession {
    pub session_id: i64,
    pub binder_id: i64,
    pub binder_title: String,
    pub mode: SessionMode,
    pub finished_at: String,
    pub total: i64,
    pub correct: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub project_count: i64,
    pub question_count: i64,
    pub answered_count: i64,
    pub due_today: i64,
    pub weak_count: i64,
    /// `None` until something has been answered — nothing attempted is not 0 % correct.
    pub accuracy: Option<f64>,
    pub recent_sessions: Vec<RecentSession>,
}

/// Everything the start page shows, in one round trip.
///
/// Every number is derived from `attempts` and `scheduling` rather than kept in a counter, so the
/// dashboard cannot drift away from the statistics view that reads the same rows.
pub fn dashboard(connection: &Connection) -> AppResult<Dashboard> {
    let project_count =
        connection.query_row("SELECT COUNT(*) FROM binders", [], |row| row.get(0))?;
    let question_count =
        connection.query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))?;
    let answered_count = connection.query_row(
        "SELECT COUNT(DISTINCT question_id) FROM attempts",
        [],
        |row| row.get(0),
    )?;
    let accuracy =
        connection.query_row("SELECT AVG(correct) FROM attempts", [], |row| row.get(0))?;
    let due_today = connection.query_row(
        "SELECT COUNT(*) FROM scheduling WHERE due_at <= datetime('now')",
        [],
        |row| row.get(0),
    )?;

    // The weak pool by the same rule the trainer uses: missed at least once, and not yet cleared
    // by two correct answers in two different sessions.
    let weak_count = connection.query_row(
        "SELECT COUNT(*) FROM questions q
         WHERE EXISTS (SELECT 1 FROM attempts a WHERE a.question_id = q.id AND a.correct = 0)
           AND NOT (
             (SELECT COUNT(*) FROM (
                SELECT a.correct FROM attempts a
                WHERE a.question_id = q.id ORDER BY a.id DESC LIMIT 2
              ) recent WHERE recent.correct = 1) = 2
             AND
             (SELECT COUNT(DISTINCT recent.session_id) FROM (
                SELECT a.session_id FROM attempts a
                WHERE a.question_id = q.id ORDER BY a.id DESC LIMIT 2
              ) recent) = 2
           )",
        [],
        |row| row.get(0),
    )?;

    let mut statement = connection.prepare(
        "SELECT s.id, s.binder_id, b.title, s.mode, s.finished_at,
                (SELECT COUNT(*) FROM attempts a WHERE a.session_id = s.id),
                (SELECT COUNT(*) FROM attempts a WHERE a.session_id = s.id AND a.correct = 1)
         FROM sessions s JOIN binders b ON b.id = s.binder_id
         WHERE s.finished_at IS NOT NULL
         ORDER BY s.finished_at DESC LIMIT 8",
    )?;
    let rows = statement
        .query_map([], |row| {
            Ok(RecentSession {
                session_id: row.get(0)?,
                binder_id: row.get(1)?,
                binder_title: row.get(2)?,
                mode: SessionMode::from_str(&row.get::<_, String>(3)?),
                finished_at: row.get(4)?,
                total: row.get(5)?,
                correct: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(Dashboard {
        project_count,
        question_count,
        answered_count,
        due_today,
        weak_count,
        accuracy,
        recent_sessions: rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::AnswerOption;
    use openexamtrainer_ingest::model::QuestionKind;

    fn question(number: u32, needs_source: bool, confidence: f32) -> QuestionDto {
        QuestionDto {
            id: 0,
            number,
            topic: Some(1),
            kind: QuestionKind::SingleChoice,
            stem: format!("Stem of question {number}"),
            options: vec![
                AnswerOption {
                    letter: 'A',
                    text: "first".into(),
                    is_correct: true,
                },
                AnswerOption {
                    letter: 'B',
                    text: "second".into(),
                    is_correct: false,
                },
            ],
            answer_letters: vec!['A'],
            matrix: Vec::new(),
            explanation: String::new(),
            references: vec![format!(
                "https://learn.microsoft.com/en-us/azure/topic-{number}"
            )],
            source_page: 1,
            confidence,
            needs_source,
            warnings: Vec::new(),
            figures: Vec::new(),
        }
    }

    fn seeded() -> (Connection, i64) {
        let mut connection = open_in_memory().expect("schema");
        let questions = vec![
            question(1, false, 1.0),
            question(2, false, 1.0),
            question(3, true, 0.65),
        ];
        let (binder_id, _) = insert_binder(
            &mut connection,
            "AI-900",
            "AI-900",
            "certleader-ai900.pdf",
            "certleader",
            &questions,
        )
        .expect("insert");
        (connection, binder_id)
    }

    #[test]
    fn every_reference_url_becomes_a_binder_bookmark() {
        let (connection, binder_id) = seeded();

        let links = list_links(&connection, binder_id).expect("links");

        assert_eq!(links.len(), 3);
        assert!(links.iter().all(|l| l.question_id.is_some()));
        assert_eq!(links[0].title, "topic 1");
    }

    #[test]
    fn a_link_title_falls_back_to_the_host_when_there_is_no_path() {
        assert_eq!(
            link_title("https://docs.microsoft.com/en-us/learn/modules/responsible-ai-principles"),
            "responsible ai principles"
        );
        assert_eq!(link_title("https://www.example.com"), "example.com");
        assert_eq!(link_title("https://example.com/a/page.html"), "page");
    }

    /// A project is created empty and filled by one import. The second import has to be refused:
    /// merging two exams into one project would average two different scores together.
    #[test]
    fn a_project_starts_empty_takes_one_import_and_refuses_a_second() {
        let mut connection = open_in_memory().expect("schema");
        let id =
            create_project(&mut connection, "Azure AI Engineer", "AI-102", "").expect("create");

        let empty = binder(&connection, id).expect("read").expect("exists");
        assert_eq!(empty.certification, "AI-102");
        assert_eq!(empty.question_count, 0);
        assert_eq!(empty.source_file, "");

        fill_project(
            &mut connection,
            id,
            "ai-102.pdf",
            "generic",
            &[question(1, false, 1.0), question(2, false, 1.0)],
        )
        .expect("fill");

        let filled = binder(&connection, id).expect("read").expect("exists");
        assert_eq!(filled.question_count, 2);
        assert_eq!(filled.source_file, "ai-102.pdf");

        let error = fill_project(
            &mut connection,
            id,
            "second.pdf",
            "generic",
            &[question(3, false, 1.0)],
        )
        .expect_err("refused");
        assert!(error.to_string().contains("one file"), "{error}");

        // The refusal has to leave the project as it was, not half-written.
        assert_eq!(
            binder(&connection, id)
                .expect("read")
                .expect("exists")
                .question_count,
            2
        );
    }

    #[test]
    fn the_dashboard_counts_nothing_as_nothing_rather_than_as_zero_percent() {
        let mut connection = open_in_memory().expect("schema");
        create_project(&mut connection, "Fresh", "AI-900", "").expect("create");

        let summary = dashboard(&connection).expect("dashboard");

        assert_eq!(summary.project_count, 1);
        assert_eq!(summary.question_count, 0);
        assert_eq!(summary.answered_count, 0);
        assert!(summary.accuracy.is_none(), "never answered is not 0 %");
        assert_eq!(summary.due_today, 0);
        assert_eq!(summary.weak_count, 0);
        assert!(summary.recent_sessions.is_empty());
    }

    #[test]
    fn a_binder_reports_the_counts_the_library_shows() {
        let (connection, _) = seeded();

        let binders = list_binders(&connection).expect("list");

        assert_eq!(binders.len(), 1);
        assert_eq!(binders[0].question_count, 3);
        assert_eq!(binders[0].needs_review_count, 1);
        assert_eq!(binders[0].needs_source_count, 1);
        assert_eq!(binders[0].attempt_count, 0);
        assert!(binders[0].accuracy.is_none());
        assert!(binders[0].last_studied_at.is_none());
    }

    #[test]
    fn review_lists_only_what_fell_below_the_threshold() {
        let (connection, binder_id) = seeded();

        let all = list_questions(&connection, binder_id, false).expect("all");
        let review = list_questions(&connection, binder_id, true).expect("review");

        assert_eq!(all.len(), 3);
        assert_eq!(review.len(), 1);
        assert_eq!(review[0].number, 3);
        assert!(review[0].needs_source);
    }

    #[test]
    fn a_scored_session_never_contains_a_question_missing_its_figure() {
        let (connection, binder_id) = seeded();

        let practice = session_questions(
            &connection,
            binder_id,
            SessionMode::Practice,
            None,
            &RuleSet::default(),
        )
        .expect("session");

        assert_eq!(practice.len(), 2);
        assert!(practice.iter().all(|q| !q.needs_source));
    }

    #[test]
    fn focus_is_derived_from_the_attempts_of_one_session() {
        let (connection, binder_id) = seeded();
        let session_id = create_session(
            &connection,
            binder_id,
            SessionMode::Practice,
            &RuleSet::default(),
        )
        .expect("id");
        let questions = session_questions(
            &connection,
            binder_id,
            SessionMode::Practice,
            None,
            &RuleSet::default(),
        )
        .expect("set");
        let wrong = questions[0].id;
        let right = questions[1].id;

        insert_attempt(&connection, session_id, wrong, "B", false, 4_000).expect("attempt");
        insert_attempt(&connection, session_id, right, "A", true, 2_000).expect("attempt");

        let totals = finish_session(&connection, session_id).expect("finish");
        assert_eq!(totals.total, 2);
        assert_eq!(totals.correct, 1);
        assert_eq!(totals.elapsed_ms, 6_000);
        assert_eq!(totals.wrong_question_ids, vec![wrong]);

        let focus = session_questions(
            &connection,
            binder_id,
            SessionMode::Focus,
            Some(session_id),
            &RuleSet::default(),
        )
        .expect("focus");
        assert_eq!(focus.iter().map(|q| q.id).collect::<Vec<_>>(), vec![wrong]);
    }

    /// One attempt per session, so "twice in a row across two sessions" is exercised honestly.
    fn answer(connection: &Connection, binder_id: i64, question_id: i64, correct: bool) {
        let session = create_session(
            connection,
            binder_id,
            SessionMode::Practice,
            &RuleSet::default(),
        )
        .expect("session");
        insert_attempt(connection, session, question_id, "A", correct, 1_000).expect("attempt");
        finish_session(connection, session).expect("finish");
    }

    #[test]
    fn a_question_enters_the_weak_pool_when_missed_and_leaves_it_after_two_clean_sessions() {
        let (connection, binder_id) = seeded_bank(3);
        let all = session_questions(
            &connection,
            binder_id,
            SessionMode::Practice,
            None,
            &RuleSet::default(),
        )
        .expect("all");
        let target = all[0].id;
        let weak = |connection: &Connection| {
            session_questions(
                connection,
                binder_id,
                SessionMode::Weak,
                None,
                &RuleSet::default(),
            )
            .expect("weak")
            .iter()
            .map(|q| q.id)
            .collect::<Vec<_>>()
        };

        assert!(weak(&connection).is_empty(), "nothing missed yet");

        answer(&connection, binder_id, target, false);
        assert_eq!(weak(&connection), vec![target]);

        answer(&connection, binder_id, target, true);
        assert_eq!(
            weak(&connection),
            vec![target],
            "one right answer is not enough"
        );

        answer(&connection, binder_id, target, true);
        assert!(weak(&connection).is_empty(), "two clean sessions retire it");
    }

    #[test]
    fn two_right_answers_inside_one_session_do_not_retire_a_question() {
        let (connection, binder_id) = seeded_bank(2);
        let target = session_questions(
            &connection,
            binder_id,
            SessionMode::Practice,
            None,
            &RuleSet::default(),
        )
        .expect("all")[0]
            .id;

        answer(&connection, binder_id, target, false);

        let session = create_session(
            &connection,
            binder_id,
            SessionMode::Practice,
            &RuleSet::default(),
        )
        .expect("session");
        insert_attempt(&connection, session, target, "A", true, 900).expect("attempt");
        insert_attempt(&connection, session, target, "A", true, 900).expect("attempt");
        finish_session(&connection, session).expect("finish");

        let weak = session_questions(
            &connection,
            binder_id,
            SessionMode::Weak,
            None,
            &RuleSet::default(),
        )
        .expect("weak");
        assert_eq!(weak.iter().map(|q| q.id).collect::<Vec<_>>(), vec![target]);
    }

    #[test]
    fn statistics_are_derived_from_the_attempt_log() {
        let (connection, binder_id) = seeded_bank(2);
        let questions = session_questions(
            &connection,
            binder_id,
            SessionMode::Practice,
            None,
            &RuleSet::default(),
        )
        .expect("all");
        let answered = questions[0].id;

        let session = create_session(
            &connection,
            binder_id,
            SessionMode::Practice,
            &RuleSet::default(),
        )
        .expect("session");
        insert_attempt(&connection, session, answered, "B", false, 4_000).expect("attempt");
        reschedule(&connection, answered, false).expect("miss");
        insert_attempt(&connection, session, answered, "A", true, 2_000).expect("attempt");
        reschedule(&connection, answered, true).expect("hit");

        let stats = question_stats(&connection, binder_id).expect("stats");
        let answered_stat = stats
            .iter()
            .find(|s| s.question_id == answered)
            .expect("stat");
        let untouched = stats
            .iter()
            .find(|s| s.question_id != answered)
            .expect("stat");

        assert_eq!(answered_stat.attempts, 2);
        assert_eq!(answered_stat.correct, 1);
        assert_eq!(answered_stat.accuracy, Some(0.5));
        assert_eq!(answered_stat.average_ms, Some(3_000.0));
        assert!(answered_stat.due_at.is_some());

        // The first miss is not a lapse: you cannot forget a question you had never got right.
        assert_eq!(answered_stat.lapses, 0);

        // An unanswered question is not 0 % — it has no accuracy at all.
        assert_eq!(untouched.attempts, 0);
        assert_eq!(untouched.accuracy, None);
        assert_eq!(untouched.average_ms, None);
        assert_eq!(untouched.lapses, 0);
    }

    /// Time is what FSRS schedules against, so a test that wants to say anything about an interval
    /// has to be able to move it. SQLite still owns the clock, exactly as it does in production.
    fn offset(connection: &Connection, modifier: &str) -> String {
        connection
            .query_row("SELECT datetime('now', ?1)", params![modifier], |row| {
                row.get(0)
            })
            .expect("stamp")
    }

    fn card_of(connection: &Connection, question_id: i64) -> Card {
        load_card(connection, question_id).expect("card")
    }

    fn first_question() -> (Connection, i64) {
        let (connection, binder_id) = seeded();
        let questions = session_questions(
            &connection,
            binder_id,
            SessionMode::Practice,
            None,
            &RuleSet::default(),
        )
        .expect("set");
        let id = questions[0].id;
        (connection, id)
    }

    /// The whole reason for leaving SM-2: the interval comes from the model's stability, and the
    /// default weights put a first correct answer three days out rather than one.
    #[test]
    fn a_first_correct_answer_schedules_from_stability() {
        let (connection, id) = first_question();

        reschedule(&connection, id, true).expect("first");
        let card = card_of(&connection, id);

        assert_eq!(card.scheduled_days, 3);
        assert_eq!(card.reps, 1);
        assert_eq!(card.lapses, 0);
        assert!((card.stability - 3.1262).abs() < 1e-9);
    }

    /// SM-2 would have jumped 1 → 6 here, because it counts repetitions. FSRS asks what was
    /// learned, and answering a question you answered a minute ago teaches nothing: retrievability
    /// is still 1, so stability does not move at all.
    ///
    /// The interval still creeps by a day, which is not growth — the scheduler keeps the four
    /// ratings in order, and Good has to land past Hard.
    #[test]
    fn a_repeat_on_the_same_day_does_not_grow_stability() {
        let (connection, id) = first_question();

        reschedule(&connection, id, true).expect("first");
        let first = card_of(&connection, id);
        reschedule(&connection, id, true).expect("second");
        let repeated = card_of(&connection, id);

        assert_eq!(repeated.stability, first.stability);
        assert_eq!(repeated.elapsed_days, 0);
        assert_eq!(repeated.reps, 2);
        assert_eq!(repeated.scheduled_days, 4);
    }

    #[test]
    fn a_correct_answer_after_the_interval_grows_it() {
        let (connection, id) = first_question();

        reschedule(&connection, id, true).expect("first");
        reschedule_at(&connection, id, true, &offset(&connection, "+3 days")).expect("second");
        let card = card_of(&connection, id);

        assert_eq!(card.elapsed_days, 3);
        assert_eq!(card.scheduled_days, 11);
        assert!(card.stability > 11.0);
    }

    #[test]
    fn a_wrong_answer_shortens_the_interval_and_counts_a_lapse() {
        let (connection, id) = first_question();

        reschedule(&connection, id, true).expect("first");
        reschedule_at(&connection, id, true, &offset(&connection, "+3 days")).expect("second");
        let learned = card_of(&connection, id);

        reschedule_at(&connection, id, false, &offset(&connection, "+14 days")).expect("lapse");
        let lapsed = card_of(&connection, id);

        assert_eq!(lapsed.lapses, 1);
        // Not a reset: reps keep counting, and the question stays in review rather than starting over.
        assert_eq!(lapsed.reps, 3);
        assert!(lapsed.scheduled_days < learned.scheduled_days);
        assert!(lapsed.stability < learned.stability);
        assert!(lapsed.difficulty > learned.difficulty);
    }

    /// The migration off SM-2 throws the old columns away rather than converting them. What makes
    /// that safe is that `attempts` never forgets, so the replay has to land on the same state the
    /// same reviews would have produced live.
    #[test]
    fn the_migration_replays_scheduling_from_the_attempt_log() {
        let (mut connection, binder_id) = seeded_bank(2);
        let questions = session_questions(
            &connection,
            binder_id,
            SessionMode::Practice,
            None,
            &RuleSet::default(),
        )
        .expect("set");
        let (first, second) = (questions[0].id, questions[1].id);
        let session = create_session(
            &connection,
            binder_id,
            SessionMode::Practice,
            &RuleSet::default(),
        )
        .expect("session");

        insert_attempt(&connection, session, first, "A", true, 1_000).expect("attempt");
        insert_attempt(&connection, session, second, "B", false, 1_000).expect("attempt");
        insert_attempt(&connection, session, first, "A", true, 1_000).expect("attempt");
        for (question_id, correct) in [(first, true), (second, false), (first, true)] {
            reschedule(&connection, question_id, correct).expect("live");
        }
        let live = [card_of(&connection, first), card_of(&connection, second)];

        connection
            .execute_batch(
                "DROP TABLE scheduling;
                 CREATE TABLE scheduling (
                     question_id   INTEGER PRIMARY KEY REFERENCES questions(id) ON DELETE CASCADE,
                     due_at        TEXT NOT NULL,
                     interval_days REAL NOT NULL,
                     ease          REAL NOT NULL,
                     reps          INTEGER NOT NULL,
                     lapses        INTEGER NOT NULL
                 );
                 INSERT INTO scheduling VALUES (1, '2020-01-01 00:00:00', 6.0, 2.5, 2, 0);",
            )
            .expect("legacy table");

        migrate_scheduling(&mut connection).expect("migrate");

        assert_eq!(
            [card_of(&connection, first), card_of(&connection, second)],
            live
        );
    }

    fn seeded_bank(count: u32) -> (Connection, i64) {
        let mut connection = open_in_memory().expect("schema");
        let questions: Vec<QuestionDto> = (1..=count).map(|n| question(n, false, 1.0)).collect();
        let (binder_id, _) = insert_binder(
            &mut connection,
            "AZ-900",
            "AZ-900",
            "bank.pdf",
            "generic",
            &questions,
        )
        .expect("insert");
        (connection, binder_id)
    }

    #[test]
    fn the_same_seed_draws_the_same_exam_and_a_different_one_does_not() {
        let (connection, binder_id) = seeded_bank(12);
        let draw = |seed: i64, count: i64| {
            session_questions(
                &connection,
                binder_id,
                SessionMode::Challenge,
                None,
                &RuleSet {
                    seed: Some(seed),
                    question_count: Some(count),
                    time_limit_seconds: None,
                },
            )
            .expect("draw")
            .iter()
            .map(|q| q.number)
            .collect::<Vec<_>>()
        };

        let first = draw(4711, 8);
        assert_eq!(first.len(), 8);
        assert_eq!(first, draw(4711, 8), "the same seed must replay exactly");
        assert_ne!(first, draw(1234, 8));

        // The draw is a shuffle, not a slice: every question is a candidate.
        let mut sorted = draw(4711, 12);
        sorted.sort_unstable();
        assert_eq!(sorted, (1..=12).collect::<Vec<_>>());
    }

    #[test]
    fn a_challenge_leaderboard_ranks_by_score_then_by_time() {
        let (connection, binder_id) = seeded_bank(4);
        let rules = RuleSet {
            seed: Some(99),
            question_count: Some(2),
            time_limit_seconds: Some(300),
        };
        let questions =
            session_questions(&connection, binder_id, SessionMode::Challenge, None, &rules)
                .expect("draw");

        let run = |correct: bool, elapsed: i64| {
            let session =
                create_session(&connection, binder_id, SessionMode::Challenge, &rules).expect("id");
            for question in &questions {
                insert_attempt(&connection, session, question.id, "A", correct, elapsed)
                    .expect("attempt");
            }
            finish_session(&connection, session).expect("finish");
            session
        };

        let slow_and_right = run(true, 9_000);
        let fast_and_right = run(true, 1_000);
        let fast_and_wrong = run(false, 500);

        let board = challenge_results(&connection, binder_id, 99).expect("board");

        assert_eq!(
            board.iter().map(|r| r.session_id).collect::<Vec<_>>(),
            vec![fast_and_right, slow_and_right, fast_and_wrong]
        );
        assert_eq!(board[0].correct, 2);
        assert_eq!(board[0].elapsed_ms, 2_000);
    }

    #[test]
    fn deleting_a_binder_takes_its_questions_with_it() {
        let (connection, binder_id) = seeded();

        delete_binder(&connection, binder_id).expect("delete");

        assert!(list_binders(&connection).expect("list").is_empty());
        let remaining: i64 = connection
            .query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))
            .expect("count");
        assert_eq!(remaining, 0);
    }
}

#[cfg(test)]
mod live {
    use super::*;

    /// Opens a copy of the real library and reads everything the overview and the exam page ask
    /// for. A migration that fails, or a query naming a column that is not there, shows up here as
    /// an error rather than as an empty window with no explanation.
    ///
    /// `LIBRARY=<path>` names the database; it is copied first, so the real one is never touched.
    #[test]
    #[ignore = "needs the app's own database"]
    fn the_real_library_still_answers_every_query() {
        let source = std::env::var("LIBRARY").expect("set LIBRARY to library.sqlite3");
        let copy = std::env::temp_dir().join("openexamtrainer-live-check.sqlite3");
        let _ = std::fs::remove_file(&copy);
        std::fs::copy(&source, &copy).expect("copy the library");

        let connection = open(&copy).expect("open");
        let binders = list_binders(&connection).expect("binders");
        let dashboard = dashboard(&connection).expect("dashboard");
        let exams = timeline(&connection).expect("timeline");
        let templates = list_templates(&connection).expect("templates");

        println!(
            "binders={} dashboard.projects={} timeline={} templates={}",
            binders.len(),
            dashboard.project_count,
            exams.len(),
            templates.len()
        );

        for binder in &binders {
            let links = list_links(&connection, binder.id).expect("links");
            let notes = list_notes(&connection, binder.id).expect("notes");
            let passed = list_certifications(&connection, binder.id).expect("certifications");
            let steps = list_progress(&connection, binder.id).expect("progress");
            println!(
                "  {} — links={} notes={} passed={} steps={} doc_url={:?}",
                binder.title,
                links.len(),
                notes.len(),
                passed.len(),
                steps.len(),
                binder.doc_url
            );
        }

        let _ = std::fs::remove_file(&copy);
    }
}

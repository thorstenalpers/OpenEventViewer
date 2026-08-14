//! The catalog, as a file.
//!
//! [`16-backend.md`](../../.agents/docs/16-backend.md) describes this surface as Supabase: Postgres
//! for the manifests, a storage bucket for the decks, RLS for who may write what. There is no
//! project, so it is a second SQLite database next to the library one and a folder of `.examdeck`
//! files next to that. The tables and their constraints mirror
//! `supabase/migrations/0001_catalog.sql` column for column; what Postgres would enforce with a
//! policy is enforced here by comparing against the local identity before the statement runs.
//!
//! The point of keeping the shapes identical is that the commands above this module do not change
//! when a real backend arrives — only this file does.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::db;
use crate::dto::{
    CatalogEntry, Identity, LeaderboardRow, QuestionDto, Rating, SyncReport, UploadPreview,
};
use crate::error::{AppError, AppResult};

const SCHEMA: &str = r#"
-- Who this machine is. One row, made on first open. It stands in for `auth.users`: a real backend
-- issues the id, this one draws it, and everything keyed on a user is keyed on this.
CREATE TABLE IF NOT EXISTS identity (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS published_binders (
    id                 TEXT PRIMARY KEY,
    owner_id           TEXT NOT NULL,
    owner_name         TEXT NOT NULL,
    title              TEXT NOT NULL,
    certification      TEXT NOT NULL,
    -- Denormalised from the deck so the catalog can sort and filter without opening the zip.
    question_count     INTEGER NOT NULL CHECK (question_count >= 0),
    needs_source_count INTEGER NOT NULL DEFAULT 0 CHECK (needs_source_count >= 0),
    profile            TEXT NOT NULL,
    -- The deck's file name inside the catalog folder, never an absolute path: the data directory
    -- moves with the profile, and a stored path would point at the old machine's disk.
    storage_path       TEXT NOT NULL UNIQUE,
    bytes              INTEGER NOT NULL DEFAULT 0,
    published_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at         TEXT NOT NULL DEFAULT (datetime('now')),
    rating_count       INTEGER NOT NULL DEFAULT 0,
    rating_sum         INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS published_binders_certification
    ON published_binders (certification);

-- One row per person per binder. Without this key a rating is a vote counter.
CREATE TABLE IF NOT EXISTS ratings (
    binder_id  TEXT NOT NULL REFERENCES published_binders (id) ON DELETE CASCADE,
    rater_id   TEXT NOT NULL,
    rater_name TEXT NOT NULL,
    stars      INTEGER NOT NULL CHECK (stars BETWEEN 1 AND 5),
    comment    TEXT NOT NULL DEFAULT '' CHECK (length(comment) <= 2000),
    rated_at   TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (binder_id, rater_id)
);

-- The aggregate is kept by these three rather than by whoever wrote the rating, so an author
-- cannot reach the two numbers the catalog sorts on.
CREATE TRIGGER IF NOT EXISTS ratings_added AFTER INSERT ON ratings
BEGIN
    UPDATE published_binders
       SET rating_count = rating_count + 1,
           rating_sum   = rating_sum + NEW.stars
     WHERE id = NEW.binder_id;
END;

CREATE TRIGGER IF NOT EXISTS ratings_changed AFTER UPDATE ON ratings
BEGIN
    UPDATE published_binders
       SET rating_sum = rating_sum - OLD.stars + NEW.stars
     WHERE id = NEW.binder_id;
END;

CREATE TRIGGER IF NOT EXISTS ratings_withdrawn AFTER DELETE ON ratings
BEGIN
    UPDATE published_binders
       SET rating_count = rating_count - 1,
           rating_sum   = rating_sum - OLD.stars
     WHERE id = OLD.binder_id;
END;

-- A posted time is a record, so nothing here updates one: a second run is a second row.
CREATE TABLE IF NOT EXISTS challenge_results (
    id             TEXT PRIMARY KEY,
    binder_id      TEXT NOT NULL REFERENCES published_binders (id) ON DELETE CASCADE,
    runner_id      TEXT NOT NULL,
    runner_name    TEXT NOT NULL,
    -- The seed is what makes two runs comparable; see db.rs `session_questions`.
    seed           INTEGER NOT NULL,
    question_count INTEGER NOT NULL CHECK (question_count > 0),
    correct        INTEGER NOT NULL CHECK (correct >= 0),
    elapsed_ms     INTEGER NOT NULL CHECK (elapsed_ms >= 0),
    finished_at    TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (correct <= question_count)
);

CREATE INDEX IF NOT EXISTS challenge_results_board
    ON challenge_results (binder_id, seed, correct DESC, elapsed_ms ASC);

-- Progress, keyed by the *content* of the question rather than by its local row id: ids are
-- per-machine, so syncing them would pair the wrong rows on the second device.
--
-- The whole FSRS card is here, not just stability and a due date. A partial card would have to be
-- completed with invented numbers on arrival, and the scheduler reads every one of them.
CREATE TABLE IF NOT EXISTS progress (
    user_id        TEXT NOT NULL,
    question_key   TEXT NOT NULL,
    attempts       INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    correct        INTEGER NOT NULL DEFAULT 0 CHECK (correct >= 0),
    due_at         TEXT,
    last_review_at TEXT,
    stability      REAL NOT NULL DEFAULT 0,
    difficulty     REAL NOT NULL DEFAULT 0,
    elapsed_days   INTEGER NOT NULL DEFAULT 0,
    scheduled_days INTEGER NOT NULL DEFAULT 0,
    reps           INTEGER NOT NULL DEFAULT 0,
    lapses         INTEGER NOT NULL DEFAULT 0,
    state          INTEGER NOT NULL DEFAULT 0,
    updated_at     TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, question_key),
    CHECK (correct <= attempts)
);
"#;

pub fn open(path: &Path) -> AppResult<Connection> {
    prepare(Connection::open(path)?)
}

fn prepare(connection: Connection) -> AppResult<Connection> {
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(SCHEMA)?;
    connection.execute(
        "INSERT INTO identity (id, name)
         SELECT lower(hex(randomblob(16))), 'local'
         WHERE NOT EXISTS (SELECT 1 FROM identity)",
        [],
    )?;
    Ok(connection)
}

/// Where a published deck lives. Content is not addressed by hash here: republishing a binder
/// rewrites the same entry, and the whole point of the entry id is that it survives the rewrite.
pub fn deck_path(data_dir: &Path, storage_path: &str) -> PathBuf {
    data_dir.join("catalog").join(storage_path)
}

pub fn identity(connection: &Connection) -> AppResult<Identity> {
    let identity = connection.query_row("SELECT id, name FROM identity", [], |row| {
        Ok(Identity {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    Ok(identity)
}

/// Renames this machine, and every entry it has already published with it — the name is display
/// text kept beside the id, and leaving old rows on the old name would show one person as two.
pub fn rename(connection: &Connection, name: &str) -> AppResult<Identity> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Message("a publisher needs a name".into()));
    }
    let id = identity(connection)?.id;
    connection.execute("UPDATE identity SET name = ?1", params![name])?;
    connection.execute(
        "UPDATE published_binders SET owner_name = ?1 WHERE owner_id = ?2",
        params![name, id],
    )?;
    connection.execute(
        "UPDATE ratings SET rater_name = ?1 WHERE rater_id = ?2",
        params![name, id],
    )?;
    connection.execute(
        "UPDATE challenge_results SET runner_name = ?1 WHERE runner_id = ?2",
        params![name, id],
    )?;
    identity(connection)
}

/// A fresh entry id, drawn before the deck is written because it names the file.
pub fn new_entry_id(connection: &Connection) -> AppResult<String> {
    Ok(connection.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))?)
}

pub struct Publication<'a> {
    pub entry_id: &'a str,
    pub title: &'a str,
    pub certification: &'a str,
    pub profile: &'a str,
    pub question_count: i64,
    pub needs_source_count: i64,
    pub bytes: i64,
}

/// Writes the manifest row for a deck already in the catalog folder.
///
/// Republishing keeps `published_at` and moves `updated_at`: the catalog sorts by the day a binder
/// first appeared, and a typo fix should not send it back to the top of the list.
pub fn publish(connection: &Connection, publication: &Publication) -> AppResult<()> {
    let owner = identity(connection)?;
    connection.execute(
        "INSERT INTO published_binders
            (id, owner_id, owner_name, title, certification, question_count, needs_source_count,
             profile, storage_path, bytes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            title              = excluded.title,
            certification      = excluded.certification,
            question_count     = excluded.question_count,
            needs_source_count = excluded.needs_source_count,
            profile            = excluded.profile,
            bytes              = excluded.bytes,
            updated_at         = datetime('now')",
        params![
            publication.entry_id,
            owner.id,
            owner.name,
            publication.title,
            publication.certification,
            publication.question_count,
            publication.needs_source_count,
            publication.profile,
            storage_path(publication.entry_id),
            publication.bytes,
        ],
    )?;
    Ok(())
}

pub fn storage_path(entry_id: &str) -> String {
    format!("{entry_id}.examdeck")
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub certification: Option<String>,
    pub search: Option<String>,
    /// `recent`, `rating`, `questions` or `title`. Anything else is `recent`.
    pub sort: Option<String>,
}

const ENTRY_COLUMNS: &str = r#"
    id, owner_id, owner_name, title, certification, profile, question_count, needs_source_count,
    bytes, published_at, updated_at, rating_count, rating_sum
"#;

fn entry_from_row(row: &rusqlite::Row<'_>, viewer: &str) -> rusqlite::Result<CatalogEntry> {
    let owner_id: String = row.get(1)?;
    let rating_count: i64 = row.get(11)?;
    let rating_sum: i64 = row.get(12)?;
    Ok(CatalogEntry {
        mine: owner_id == viewer,
        id: row.get(0)?,
        owner_id,
        owner_name: row.get(2)?,
        title: row.get(3)?,
        certification: row.get(4)?,
        profile: row.get(5)?,
        question_count: row.get(6)?,
        needs_source_count: row.get(7)?,
        bytes: row.get(8)?,
        published_at: row.get(9)?,
        updated_at: row.get(10)?,
        rating_count,
        // No rating at all is not nought stars, the same way an unanswered question is not 0 %.
        rating: (rating_count > 0).then(|| rating_sum as f64 / rating_count as f64),
    })
}

/// The catalog listing, filtered and sorted in SQL rather than in the view — the same division a
/// Postgres-backed catalog needs, so the table above it does not change when one arrives.
pub fn list(connection: &Connection, filter: &Filter) -> AppResult<Vec<CatalogEntry>> {
    let viewer = identity(connection)?.id;
    let order = match filter.sort.as_deref() {
        Some("rating") => {
            "CASE WHEN rating_count = 0 THEN 1 ELSE 0 END,
             CAST(rating_sum AS REAL) / MAX(rating_count, 1) DESC, title"
        }
        Some("questions") => "question_count DESC, title",
        Some("title") => "title COLLATE NOCASE",
        _ => "published_at DESC, title",
    };
    let sql = format!(
        "SELECT {ENTRY_COLUMNS} FROM published_binders
         WHERE (?1 IS NULL OR certification = ?1)
           AND (?2 IS NULL OR title LIKE '%' || ?2 || '%' COLLATE NOCASE
                           OR certification LIKE '%' || ?2 || '%' COLLATE NOCASE)
         ORDER BY {order}"
    );
    let certification = filter.certification.as_deref().filter(|s| !s.is_empty());
    let search = filter.search.as_deref().filter(|s| !s.is_empty());
    let mut statement = connection.prepare(&sql)?;
    let entries = statement
        .query_map(params![certification, search], |row| {
            entry_from_row(row, &viewer)
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(entries)
}

pub fn entry(connection: &Connection, entry_id: &str) -> AppResult<CatalogEntry> {
    let viewer = identity(connection)?.id;
    let sql = format!("SELECT {ENTRY_COLUMNS} FROM published_binders WHERE id = ?1");
    connection
        .query_row(&sql, params![entry_id], |row| entry_from_row(row, &viewer))
        .optional()?
        .ok_or_else(|| AppError::Message(format!("no catalog entry {entry_id}")))
}

/// Removes an entry and the deck behind it. Only the owner may: the check Postgres would do in a
/// `USING` clause happens here, because there is no policy engine under this connection.
pub fn withdraw(connection: &Connection, data_dir: &Path, entry_id: &str) -> AppResult<()> {
    let entry = entry(connection, entry_id)?;
    if !entry.mine {
        return Err(AppError::Message(
            "only the publisher may withdraw a binder".into(),
        ));
    }
    connection.execute(
        "DELETE FROM published_binders WHERE id = ?1",
        params![entry_id],
    )?;
    std::fs::remove_file(deck_path(data_dir, &storage_path(entry_id))).ok();
    Ok(())
}

pub fn rate(
    connection: &Connection,
    entry_id: &str,
    stars: i64,
    comment: &str,
) -> AppResult<Vec<Rating>> {
    if !(1..=5).contains(&stars) {
        return Err(AppError::Message(format!(
            "a rating is one to five stars, not {stars}"
        )));
    }
    let rater = identity(connection)?;
    connection.execute(
        "INSERT INTO ratings (binder_id, rater_id, rater_name, stars, comment)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(binder_id, rater_id) DO UPDATE SET
            stars    = excluded.stars,
            comment  = excluded.comment,
            rated_at = datetime('now')",
        params![entry_id, rater.id, rater.name, stars, comment],
    )?;
    ratings(connection, entry_id)
}

pub fn ratings(connection: &Connection, entry_id: &str) -> AppResult<Vec<Rating>> {
    let viewer = identity(connection)?.id;
    let mut statement = connection.prepare(
        "SELECT rater_id, rater_name, stars, comment, rated_at
         FROM ratings WHERE binder_id = ?1 ORDER BY rated_at DESC",
    )?;
    let ratings = statement
        .query_map(params![entry_id], |row| {
            let rater_id: String = row.get(0)?;
            Ok(Rating {
                mine: rater_id == viewer,
                rater_id,
                rater_name: row.get(1)?,
                stars: row.get(2)?,
                comment: row.get(3)?,
                rated_at: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(ratings)
}

pub fn post_result(
    connection: &Connection,
    entry_id: &str,
    seed: i64,
    question_count: i64,
    correct: i64,
    elapsed_ms: i64,
) -> AppResult<Vec<LeaderboardRow>> {
    let runner = identity(connection)?;
    let id = new_entry_id(connection)?;
    connection.execute(
        "INSERT INTO challenge_results
            (id, binder_id, runner_id, runner_name, seed, question_count, correct, elapsed_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            entry_id,
            runner.id,
            runner.name,
            seed,
            question_count,
            correct,
            elapsed_ms
        ],
    )?;
    leaderboard(connection, entry_id, seed)
}

/// One seed, one board: same questions in the same order, so accuracy and then time compare.
pub fn leaderboard(
    connection: &Connection,
    entry_id: &str,
    seed: i64,
) -> AppResult<Vec<LeaderboardRow>> {
    let viewer = identity(connection)?.id;
    let mut statement = connection.prepare(
        "SELECT runner_id, runner_name, seed, question_count, correct, elapsed_ms, finished_at
         FROM challenge_results
         WHERE binder_id = ?1 AND seed = ?2
         ORDER BY correct DESC, elapsed_ms ASC",
    )?;
    let rows = statement
        .query_map(params![entry_id, seed], |row| {
            let runner_id: String = row.get(0)?;
            Ok(LeaderboardRow {
                mine: runner_id == viewer,
                runner_id,
                runner_name: row.get(1)?,
                seed: row.get(2)?,
                question_count: row.get(3)?,
                correct: row.get(4)?,
                elapsed_ms: row.get(5)?,
                finished_at: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// The seeds this entry has a board for, so the view can offer them instead of asking for a number.
pub fn seeds(connection: &Connection, entry_id: &str) -> AppResult<Vec<i64>> {
    let mut statement = connection.prepare(
        "SELECT DISTINCT seed FROM challenge_results WHERE binder_id = ?1 ORDER BY seed",
    )?;
    let seeds = statement
        .query_map(params![entry_id], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(seeds)
}

/// What identifies a question across two machines: its stem and its answer key.
///
/// Not the row id, which is per-machine, and not the number, which repeats within a single dump.
/// Whitespace is collapsed first so a re-import under a slightly different extractor still lands on
/// the same key.
pub fn question_key(question: &QuestionDto) -> String {
    let stem: Vec<&str> = question.stem.split_whitespace().collect();
    let mut letters: Vec<String> = question
        .answer_letters
        .iter()
        .map(|letter| letter.to_ascii_uppercase().to_string())
        .collect();
    letters.sort();

    let mut hasher = Sha256::new();
    hasher.update(stem.join(" ").as_bytes());
    hasher.update(b"\n");
    hasher.update(letters.join(",").as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Sends every answered question up.
///
/// A stored row with more attempts than the local one is not overwritten. `attempts` is
/// append-only, so a lower count is an older state whatever its clock says — which makes this
/// safe against the two machines' clocks disagreeing, where a plain last-write-wins is not.
pub fn push(catalog: &Connection, library: &Connection) -> AppResult<SyncReport> {
    let user_id = identity(catalog)?.id;
    let mut pushed = 0;
    let mut skipped = 0;

    for (question, attempts, correct) in db::questions_with_attempts(library)? {
        if attempts == 0 {
            continue;
        }
        let key = question_key(&question);
        let card = db::scheduling(library, question.id)?;
        let changed = catalog.execute(
            "INSERT INTO progress
                (user_id, question_key, attempts, correct, due_at, last_review_at, stability,
                 difficulty, elapsed_days, scheduled_days, reps, lapses, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(user_id, question_key) DO UPDATE SET
                attempts       = excluded.attempts,
                correct        = excluded.correct,
                due_at         = excluded.due_at,
                last_review_at = excluded.last_review_at,
                stability      = excluded.stability,
                difficulty     = excluded.difficulty,
                elapsed_days   = excluded.elapsed_days,
                scheduled_days = excluded.scheduled_days,
                reps           = excluded.reps,
                lapses         = excluded.lapses,
                state          = excluded.state,
                updated_at     = datetime('now')
             WHERE excluded.attempts > progress.attempts",
            params![
                user_id,
                key,
                attempts,
                correct,
                card.as_ref().map(|c| c.due_at.clone()),
                card.as_ref().map(|c| c.last_review_at.clone()),
                card.as_ref().map_or(0.0, |c| c.stability),
                card.as_ref().map_or(0.0, |c| c.difficulty),
                card.as_ref().map_or(0, |c| c.elapsed_days),
                card.as_ref().map_or(0, |c| c.scheduled_days),
                card.as_ref().map_or(0, |c| c.reps),
                card.as_ref().map_or(0, |c| c.lapses),
                card.as_ref().map_or(0, |c| c.state),
            ],
        )?;
        if changed > 0 {
            pushed += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(SyncReport {
        pushed,
        pulled: 0,
        skipped,
    })
}

/// Brings a stored card down onto the questions that match it by key.
///
/// Only the schedule crosses: `attempts` here is an aggregate, and the local table it would have to
/// land in is append-only, so writing rows into it to make the counts agree would be forging a
/// history. What arrives is when the question is next due and what the scheduler believes about it.
///
/// Which is why a second pull has to be judged on the card and not on the counts: the local
/// `attempts` never rises to meet the stored one, so a pull that only compared those two would
/// report the same row as applied every time it ran.
pub fn pull(catalog: &Connection, library: &Connection) -> AppResult<SyncReport> {
    let user_id = identity(catalog)?.id;
    let mut stored = catalog.prepare(
        "SELECT question_key, attempts, due_at, last_review_at, stability, difficulty,
                elapsed_days, scheduled_days, reps, lapses, state
         FROM progress WHERE user_id = ?1",
    )?;
    let rows = stored
        .query_map(params![user_id], |row| {
            let due_at: Option<String> = row.get(2)?;
            let last_review_at: Option<String> = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                due_at
                    .zip(last_review_at)
                    .map(|(due_at, last_review_at)| db::Scheduling {
                        due_at,
                        last_review_at,
                        stability: row.get(4).unwrap_or_default(),
                        difficulty: row.get(5).unwrap_or_default(),
                        elapsed_days: row.get(6).unwrap_or_default(),
                        scheduled_days: row.get(7).unwrap_or_default(),
                        reps: row.get(8).unwrap_or_default(),
                        lapses: row.get(9).unwrap_or_default(),
                        state: row.get(10).unwrap_or_default(),
                    }),
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut pulled = 0;
    let mut skipped = 0;
    for (question, attempts, _) in db::questions_with_attempts(library)? {
        let key = question_key(&question);
        let Some((_, remote_attempts, Some(card))) =
            rows.iter().find(|(stored, _, _)| *stored == key)
        else {
            continue;
        };
        let here = db::scheduling(library, question.id)?;
        let newer = here
            .as_ref()
            .is_none_or(|here| card.last_review_at > here.last_review_at);
        if *remote_attempts > attempts && newer {
            db::put_scheduling(library, question.id, card)?;
            pulled += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(SyncReport {
        pushed: 0,
        pulled,
        skipped,
    })
}

/// What publishing would put in the catalog, measured off the deck it actually writes rather than
/// counted off the tables — [hard rule 2](../../AGENTS.md) is a preview of what leaves the machine,
/// and a preview assembled a second way is a preview of something else.
pub struct Contents {
    pub questions: usize,
    pub links: usize,
    pub videos: usize,
    pub notes: usize,
    pub figures: usize,
    pub bytes: i64,
}

pub fn preview(title: &str, certification: &str, contents: &Contents) -> UploadPreview {
    UploadPreview {
        title: title.to_string(),
        certification: certification.to_string(),
        question_count: contents.questions as i64,
        link_count: contents.links as i64,
        video_count: contents.videos as i64,
        note_count: contents.notes as i64,
        figure_count: contents.figures as i64,
        bytes: contents.bytes,
        // The zip carries no `sources/` folder: `deck::export` does not write one, so the original
        // PDF stays on this machine whatever the author chose to import.
        includes_source: false,
    }
}

#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    prepare(Connection::open_in_memory()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::AnswerOption;
    use openexamtrainer_ingest::model::QuestionKind;

    fn question(number: u32, stem: &str) -> QuestionDto {
        QuestionDto {
            id: 0,
            number,
            topic: Some(1),
            kind: QuestionKind::SingleChoice,
            stem: stem.to_string(),
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
            references: Vec::new(),
            source_page: 1,
            confidence: 1.0,
            needs_source: false,
            warnings: Vec::new(),
            figures: Vec::new(),
        }
    }

    fn publish_one(catalog: &Connection, title: &str) -> String {
        let entry_id = new_entry_id(catalog).expect("id");
        publish(
            catalog,
            &Publication {
                entry_id: &entry_id,
                title,
                certification: "AI-900",
                profile: "certleader",
                question_count: 11,
                needs_source_count: 0,
                bytes: 4096,
            },
        )
        .expect("publish");
        entry_id
    }

    #[test]
    fn republishing_replaces_the_entry_it_already_has() {
        let catalog = open_in_memory().expect("schema");
        let entry_id = publish_one(&catalog, "AI-900");
        publish(
            &catalog,
            &Publication {
                entry_id: &entry_id,
                title: "AI-900 (fixed)",
                certification: "AI-900",
                profile: "certleader",
                question_count: 12,
                needs_source_count: 1,
                bytes: 5000,
            },
        )
        .expect("republish");

        let entries = list(&catalog, &Filter::default()).expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "AI-900 (fixed)");
        assert_eq!(entries[0].question_count, 12);
        assert!(entries[0].mine);
    }

    #[test]
    fn the_aggregate_follows_the_rating_rather_than_the_author() {
        let catalog = open_in_memory().expect("schema");
        let entry_id = publish_one(&catalog, "AI-900");

        rate(&catalog, &entry_id, 4, "solid").expect("rate");
        assert_eq!(entry(&catalog, &entry_id).expect("entry").rating, Some(4.0));

        // The same person rating again is a correction, not a second vote.
        let ratings = rate(&catalog, &entry_id, 2, "changed my mind").expect("rate");
        assert_eq!(ratings.len(), 1);
        assert!(ratings[0].mine);
        let entry = entry(&catalog, &entry_id).expect("entry");
        assert_eq!(entry.rating_count, 1);
        assert_eq!(entry.rating, Some(2.0));
    }

    #[test]
    fn an_unrated_binder_reports_no_rating_rather_than_nought() {
        let catalog = open_in_memory().expect("schema");
        publish_one(&catalog, "AI-900");
        assert_eq!(
            list(&catalog, &Filter::default()).expect("list")[0].rating,
            None
        );
    }

    #[test]
    fn the_board_ranks_by_score_then_by_time() {
        let catalog = open_in_memory().expect("schema");
        let entry_id = publish_one(&catalog, "AI-900");

        post_result(&catalog, &entry_id, 7, 10, 8, 90_000).expect("post");
        post_result(&catalog, &entry_id, 7, 10, 9, 120_000).expect("post");
        let board = post_result(&catalog, &entry_id, 7, 10, 8, 60_000).expect("post");

        assert_eq!(
            board
                .iter()
                .map(|r| (r.correct, r.elapsed_ms))
                .collect::<Vec<_>>(),
            vec![(9, 120_000), (8, 60_000), (8, 90_000)]
        );
        // A different seed is a different exam and never shares a board.
        assert!(leaderboard(&catalog, &entry_id, 8)
            .expect("board")
            .is_empty());
        assert_eq!(seeds(&catalog, &entry_id).expect("seeds"), vec![7]);
    }

    #[test]
    fn only_the_publisher_may_withdraw() {
        let catalog = open_in_memory().expect("schema");
        let entry_id = publish_one(&catalog, "AI-900");
        catalog
            .execute(
                "UPDATE published_binders SET owner_id = 'someone-else' WHERE id = ?1",
                params![entry_id],
            )
            .expect("reassign");

        let error = withdraw(&catalog, Path::new("."), &entry_id).expect_err("refused");
        assert!(error.to_string().contains("publisher"), "{error}");
    }

    #[test]
    fn the_key_survives_a_reimport_and_separates_two_questions() {
        let first = question(1, "Which  service\nstores blobs?");
        let same = question(9, "Which service stores blobs?");
        let other = question(1, "Which service stores tables?");

        assert_eq!(question_key(&first), question_key(&same));
        assert_ne!(question_key(&first), question_key(&other));

        // The answer key is part of the identity: the same stem with a different answer is a
        // different question, and drilling it against the old schedule would be drilling a lie.
        let mut rekeyed = question(1, "Which service stores blobs?");
        rekeyed.answer_letters = vec!['B'];
        assert_ne!(question_key(&same), question_key(&rekeyed));
    }

    /// Two libraries, one catalog: the shape a second machine has, without a second machine.
    #[test]
    fn a_schedule_reaches_the_other_library_by_question_key() {
        let catalog = open_in_memory().expect("schema");
        let mut here = db::open_in_memory().expect("schema");
        let mut there = db::open_in_memory().expect("schema");

        let questions = vec![question(1, "Which service stores blobs?")];
        let (binder_here, ids_here) = db::insert_binder(
            &mut here, "AZ-900", "AZ-900", "a.pdf", "generic", &questions,
        )
        .expect("insert");

        // The other machine studied something else first, so the same question sits on a different
        // row id there. Matching by id would pair the wrong two rows and this is what proves it.
        db::insert_binder(
            &mut there,
            "AI-900",
            "AI-900",
            "other.pdf",
            "generic",
            &[question(1, "Which service recognises faces?")],
        )
        .expect("decoy");
        let (_, ids_there) = db::insert_binder(
            &mut there,
            "AZ-900",
            "AZ-900",
            "az900-copy.pdf",
            "generic",
            &questions,
        )
        .expect("insert");
        assert_ne!(ids_here[0], ids_there[0]);

        let session = db::create_session(
            &here,
            binder_here,
            crate::dto::SessionMode::Practice,
            &db::RuleSet::default(),
        )
        .expect("session");
        db::insert_attempt(&here, session, ids_here[0], "A", true, 4_000).expect("attempt");
        db::reschedule(&here, ids_here[0], true).expect("reschedule");

        let pushed = push(&catalog, &here).expect("push");
        assert_eq!(pushed.pushed, 1);

        let pulled = pull(&catalog, &there).expect("pull");
        assert_eq!(pulled.pulled, 1);

        let landed = db::scheduling(&there, ids_there[0])
            .expect("card")
            .expect("a card");
        let original = db::scheduling(&here, ids_here[0])
            .expect("card")
            .expect("a card");
        assert_eq!(landed.due_at, original.due_at);
        assert_eq!(landed.stability, original.stability);
        assert_eq!(landed.reps, original.reps);

        // Pulling again changes nothing: the local question is now level with the stored one.
        assert_eq!(pull(&catalog, &there).expect("pull").pulled, 0);
    }

    #[test]
    fn a_thinner_history_does_not_overwrite_a_richer_one() {
        let catalog = open_in_memory().expect("schema");
        let mut busy = db::open_in_memory().expect("schema");
        let mut idle = db::open_in_memory().expect("schema");
        let questions = vec![question(1, "Which service stores blobs?")];

        for (library, attempts) in [(&mut busy, 3), (&mut idle, 1)] {
            let (binder, ids) =
                db::insert_binder(library, "AZ-900", "AZ-900", "a.pdf", "generic", &questions)
                    .expect("insert");
            let session = db::create_session(
                library,
                binder,
                crate::dto::SessionMode::Practice,
                &db::RuleSet::default(),
            )
            .expect("session");
            for _ in 0..attempts {
                db::insert_attempt(library, session, ids[0], "A", true, 1_000).expect("attempt");
            }
        }

        assert_eq!(push(&catalog, &busy).expect("push").pushed, 1);
        let thin = push(&catalog, &idle).expect("push");
        assert_eq!(thin.pushed, 0);
        assert_eq!(thin.skipped, 1);

        let stored: i64 = catalog
            .query_row("SELECT attempts FROM progress", [], |row| row.get(0))
            .expect("row");
        assert_eq!(stored, 3);
    }

    /// The whole loop the catalog exists for: a binder leaves one library as a deck, its manifest
    /// lands in the catalog, and a second library takes it back out. Nothing here goes near a
    /// network, but every step is the one a networked catalog would take.
    #[test]
    fn a_published_binder_comes_back_out_of_the_catalog() {
        let catalog = open_in_memory().expect("schema");
        let mut author = db::open_in_memory().expect("schema");
        let mut reader = db::open_in_memory().expect("schema");

        let data_dir = std::env::temp_dir().join("oet-catalog-roundtrip");
        let _ = std::fs::remove_dir_all(&data_dir);
        std::fs::create_dir_all(data_dir.join("catalog")).expect("store");

        let questions = vec![
            question(1, "Which service stores blobs?"),
            question(2, "Which service runs containers?"),
        ];
        let (binder_id, _) = db::insert_binder(
            &mut author,
            "AZ-900",
            "AZ-900",
            "certshared-az900.pdf",
            "certshared",
            &questions,
        )
        .expect("insert");

        let entry_id = new_entry_id(&catalog).expect("id");
        let path = deck_path(&data_dir, &storage_path(&entry_id));
        crate::deck::export(&author, binder_id, &data_dir, &path).expect("export");
        let bytes = std::fs::metadata(&path).expect("deck").len() as i64;
        publish(
            &catalog,
            &Publication {
                entry_id: &entry_id,
                title: "AZ-900",
                certification: "AZ-900",
                profile: "certshared",
                question_count: 2,
                needs_source_count: 0,
                bytes,
            },
        )
        .expect("publish");

        let listed = list(&catalog, &Filter::default()).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].bytes, bytes);
        assert!(bytes > 0, "the catalog reports the size of a real file");

        let imported =
            crate::deck::import(&mut reader, &data_dir, &path).expect("import from the catalog");
        assert_eq!(imported.title, "AZ-900");
        assert_eq!(imported.question_count, 2);
        assert_eq!(
            db::list_questions(&reader, imported.id, false)
                .expect("questions")
                .iter()
                .map(|q| q.stem.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Which service stores blobs?",
                "Which service runs containers?"
            ]
        );

        // Withdrawing takes the deck with it: an entry nobody can list must not leave a file behind
        // that the next publish would silently reuse.
        withdraw(&catalog, &data_dir, &entry_id).expect("withdraw");
        assert!(list(&catalog, &Filter::default()).expect("list").is_empty());
        assert!(!path.exists());

        std::fs::remove_dir_all(&data_dir).ok();
    }

    #[test]
    fn a_rename_moves_every_row_that_carries_the_old_one() {
        let catalog = open_in_memory().expect("schema");
        let entry_id = publish_one(&catalog, "AI-900");
        rate(&catalog, &entry_id, 5, "").expect("rate");
        post_result(&catalog, &entry_id, 1, 10, 10, 1_000).expect("post");

        rename(&catalog, "thorsten").expect("rename");

        assert_eq!(
            entry(&catalog, &entry_id).expect("entry").owner_name,
            "thorsten"
        );
        assert_eq!(
            ratings(&catalog, &entry_id).expect("ratings")[0].rater_name,
            "thorsten"
        );
        assert_eq!(
            leaderboard(&catalog, &entry_id, 1).expect("board")[0].runner_name,
            "thorsten"
        );
        assert!(rename(&catalog, "   ").is_err());
    }
}

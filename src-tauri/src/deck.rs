use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;

use crate::db;
use crate::dto::{Binder, Note, QuestionDto, Video};
use crate::error::{AppError, AppResult};

pub const FORMAT: &str = "examdeck/1";

const MANIFEST: &str = "manifest.json";
const QUESTIONS: &str = "questions.json";
const LINKS: &str = "links.json";
const MEDIA: &str = "media.json";
const NOTES: &str = "notes.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub format: String,
    pub title: String,
    pub certification: String,
    pub source_file: String,
    pub profile: String,
    pub question_count: usize,
    pub exported_at: String,
}

/// Attachments reference a question by its position in `questions.json`, not by a database id.
/// Ids are local to one machine, and question numbers are not unique — these dumps contain
/// duplicates — so position is the only anchor that survives a round trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckLink {
    pub question_index: Option<usize>,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckVideo {
    pub question_index: Option<usize>,
    pub url: String,
    pub title: String,
    pub start_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckNote {
    pub question_index: Option<usize>,
    pub body_md: String,
}

/// Where a deck keeps its captured figures, named by the same content hash the questions carry.
const FIGURES: &str = "figures/";

pub fn export(
    connection: &Connection,
    binder_id: i64,
    data_dir: &Path,
    destination: &Path,
) -> AppResult<Manifest> {
    let binder = db::binder(connection, binder_id)?
        .ok_or_else(|| AppError::Message(format!("no binder {binder_id}")))?;
    let questions = db::list_questions(connection, binder_id, false)?;
    let position: HashMap<i64, usize> = questions
        .iter()
        .enumerate()
        .map(|(index, question)| (question.id, index))
        .collect();

    let links: Vec<DeckLink> = db::list_links(connection, binder_id)?
        .into_iter()
        .map(|link| DeckLink {
            question_index: link.question_id.and_then(|id| position.get(&id).copied()),
            url: link.url,
            title: link.title,
        })
        .collect();
    let videos: Vec<DeckVideo> = db::list_videos(connection, binder_id)?
        .into_iter()
        .map(|video| DeckVideo {
            question_index: video.question_id.and_then(|id| position.get(&id).copied()),
            url: video.url,
            title: video.title,
            start_seconds: video.start_seconds,
        })
        .collect();
    let notes: Vec<DeckNote> = db::list_notes(connection, binder_id)?
        .into_iter()
        .map(|note| DeckNote {
            question_index: note.question_id.and_then(|id| position.get(&id).copied()),
            body_md: note.body_md,
        })
        .collect();

    let manifest = Manifest {
        format: FORMAT.to_string(),
        title: binder.title,
        certification: binder.certification,
        source_file: binder.source_file,
        profile: binder.profile,
        question_count: questions.len(),
        exported_at: db::now(connection)?,
    };

    let file = std::fs::File::create(destination)
        .map_err(|error| AppError::Message(format!("cannot write {destination:?}: {error}")))?;
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    write_json(&mut writer, MANIFEST, &manifest, options)?;
    write_json(&mut writer, QUESTIONS, &questions, options)?;
    write_json(&mut writer, LINKS, &links, options)?;
    write_json(&mut writer, MEDIA, &videos, options)?;
    write_json(&mut writer, NOTES, &notes, options)?;

    // Without the pixels a deck's figure hashes resolve to nothing on the machine that opens it,
    // and half the questions become unanswerable again on the way out of this one.
    let mut written: Vec<&str> = Vec::new();
    for hash in questions.iter().flat_map(|question| &question.figures) {
        if written.contains(&hash.as_str()) {
            continue;
        }
        let Ok(png) = std::fs::read(crate::commands::figure_path(data_dir, hash)) else {
            continue;
        };
        writer
            .start_file(format!("{FIGURES}{hash}.png"), options)
            .map_err(|error| AppError::Message(error.to_string()))?;
        writer.write_all(&png)?;
        written.push(hash);
    }

    writer
        .finish()
        .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(manifest)
}

pub fn import(connection: &mut Connection, data_dir: &Path, source: &Path) -> AppResult<Binder> {
    let file = std::fs::File::open(source)
        .map_err(|error| AppError::Message(format!("cannot read {source:?}: {error}")))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| AppError::Message(error.to_string()))?;

    let manifest: Manifest = read_json(&mut archive, MANIFEST)?;
    if manifest.format != FORMAT {
        return Err(AppError::Message(format!(
            "unsupported deck format {} — this build reads {FORMAT}",
            manifest.format
        )));
    }

    let questions: Vec<QuestionDto> = read_json(&mut archive, QUESTIONS)?;
    let links: Vec<DeckLink> = read_json(&mut archive, LINKS)?;
    let videos: Vec<DeckVideo> = read_json(&mut archive, MEDIA)?;
    let notes: Vec<DeckNote> = read_json(&mut archive, NOTES)?;

    // The figures are content-addressed, so unpacking them into the shared store is idempotent:
    // two decks that quote the same answer area end up sharing one file.
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::Message(error.to_string()))?;
        let Some(hash) = entry
            .enclosed_name()
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".png"))
            .filter(|name| name.len() == 64 && name.chars().all(|c| c.is_ascii_hexdigit()))
            .map(str::to_string)
        else {
            continue;
        };
        let destination = crate::commands::figure_path(data_dir, &hash);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut png = Vec::new();
        entry.read_to_end(&mut png)?;
        std::fs::write(destination, png)?;
    }

    let (binder_id, question_ids) = db::insert_binder(
        connection,
        &manifest.title,
        &manifest.certification,
        &manifest.source_file,
        &manifest.profile,
        &questions,
    )?;
    let anchor = |index: Option<usize>| index.and_then(|i| question_ids.get(i).copied());

    for link in links {
        db::insert_link(
            connection,
            binder_id,
            anchor(link.question_index),
            &link.url,
            &link.title,
        )?;
    }
    for video in videos {
        db::insert_video(
            connection,
            binder_id,
            &Video {
                id: 0,
                question_id: anchor(video.question_index),
                url: video.url,
                title: video.title,
                start_seconds: video.start_seconds,
            },
        )?;
    }
    for note in notes {
        db::upsert_note(
            connection,
            binder_id,
            &Note {
                id: 0,
                question_id: anchor(note.question_index),
                body_md: note.body_md,
                updated_at: String::new(),
            },
        )?;
    }

    db::binder(connection, binder_id)?
        .ok_or_else(|| AppError::Message("the binder vanished right after import".into()))
}

/// Reads only the manifest, for the catalog listing and for the import preview.
pub fn peek(source: &Path) -> AppResult<Manifest> {
    let file = std::fs::File::open(source)
        .map_err(|error| AppError::Message(format!("cannot read {source:?}: {error}")))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| AppError::Message(error.to_string()))?;
    read_json(&mut archive, MANIFEST)
}

fn write_json<W: Write + Seek, T: Serialize>(
    writer: &mut zip::ZipWriter<W>,
    name: &str,
    value: &T,
    options: SimpleFileOptions,
) -> AppResult<()> {
    writer
        .start_file(name, options)
        .map_err(|error| AppError::Message(error.to_string()))?;
    writer
        .write_all(serde_json::to_string_pretty(value)?.as_bytes())
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(())
}

fn read_json<R: Read + Seek, T: for<'de> Deserialize<'de>>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> AppResult<T> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| AppError::Message(format!("the deck has no {name}")))?;
    let mut raw = String::new();
    entry
        .read_to_string(&mut raw)
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(serde_json::from_str(&raw)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{AnswerOption, SessionMode};
    use openexamtrainer_ingest::model::QuestionKind;

    fn question(number: u32) -> QuestionDto {
        QuestionDto {
            id: 0,
            number,
            topic: Some(1),
            kind: QuestionKind::SingleChoice,
            stem: format!("Stem {number}"),
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
            explanation: "because".into(),
            references: vec![format!("https://learn.microsoft.com/azure/topic-{number}")],
            source_page: 1,
            confidence: 1.0,
            needs_source: false,
            warnings: Vec::new(),
            figures: Vec::new(),
        }
    }

    const FIGURE_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const FIGURE_PNG: &[u8] = b"not really a png, but the bytes have to survive verbatim";

    /// A scratch pair of stores, so an export and the import that follows it never share a
    /// directory — otherwise a figure "surviving" would prove nothing.
    fn stores(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("oet-deck-{name}"));
        let _ = std::fs::remove_dir_all(&root);
        (root.join("from"), root.join("to"))
    }

    #[test]
    fn a_binder_survives_the_round_trip_with_its_attachments() {
        let mut source = db::open_in_memory().expect("schema");
        let mut questions = vec![question(1), question(2)];
        questions[0].figures = vec![FIGURE_HASH.to_string()];
        let (binder_id, question_ids) = db::insert_binder(
            &mut source,
            "AI-900",
            "AI-900",
            "certleader-ai900.pdf",
            "certleader",
            &questions,
        )
        .expect("insert");

        db::insert_video(
            &source,
            binder_id,
            &Video {
                id: 0,
                question_id: Some(question_ids[1]),
                url: "https://www.youtube.com/watch?v=abc".into(),
                title: "Clustering in four minutes".into(),
                start_seconds: 145,
            },
        )
        .expect("video");
        db::upsert_note(
            &source,
            binder_id,
            &Note {
                id: 0,
                question_id: Some(question_ids[0]),
                body_md: "Transparency is about documentation.".into(),
                updated_at: String::new(),
            },
        )
        .expect("note");

        let (from, to) = stores("roundtrip");
        std::fs::create_dir_all(from.join("figures")).expect("store");
        std::fs::write(crate::commands::figure_path(&from, FIGURE_HASH), FIGURE_PNG)
            .expect("figure");

        let path = std::env::temp_dir().join("openexamtrainer-roundtrip.examdeck");
        let manifest = export(&source, binder_id, &from, &path).expect("export");
        assert_eq!(manifest.format, FORMAT);
        assert_eq!(manifest.question_count, 2);
        assert_eq!(peek(&path).expect("peek").title, "AI-900");

        let mut target = db::open_in_memory().expect("schema");
        let imported = import(&mut target, &to, &path).expect("import");

        assert_eq!(imported.title, "AI-900");
        assert_eq!(imported.question_count, 2);

        // A deck that quotes a figure has to carry the pixels, or the question it belongs to is
        // unanswerable again on the machine that opens it.
        assert_eq!(
            std::fs::read(crate::commands::figure_path(&to, FIGURE_HASH)).expect("figure"),
            FIGURE_PNG
        );

        let restored = db::list_questions(&target, imported.id, false).expect("questions");
        assert_eq!(
            restored.iter().map(|q| q.stem.as_str()).collect::<Vec<_>>(),
            vec!["Stem 1", "Stem 2"]
        );
        assert!(restored[0].options[0].is_correct);
        assert_eq!(restored[0].figures, vec![FIGURE_HASH.to_string()]);

        let videos = db::list_videos(&target, imported.id).expect("videos");
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].start_seconds, 145);
        assert_eq!(videos[0].question_id, Some(restored[1].id));

        let notes = db::list_notes(&target, imported.id).expect("notes");
        assert_eq!(notes[0].question_id, Some(restored[0].id));

        // Two per question: the extractor's reference, re-created on insert, plus the copy the
        // deck carried. The unique index collapses them, which is what keeps a round trip stable.
        let links = db::list_links(&target, imported.id).expect("links");
        assert_eq!(links.len(), 2);

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir_all(from.parent().expect("root")).ok();
    }

    #[test]
    fn a_foreign_format_is_refused_by_name() {
        let mut source = db::open_in_memory().expect("schema");
        let (binder_id, _) = db::insert_binder(
            &mut source,
            "AI-900",
            "AI-900",
            "x.pdf",
            "generic",
            &[question(1)],
        )
        .expect("insert");
        let scratch = std::env::temp_dir().join("oet-deck-foreign");
        let path = std::env::temp_dir().join("openexamtrainer-foreign.examdeck");
        export(&source, binder_id, &scratch, &path).expect("export");

        // Rewrite the manifest with a format this build does not know.
        let mut manifest = peek(&path).expect("peek");
        manifest.format = "examdeck/99".into();
        rewrite_manifest(&path, &manifest);

        let mut target = db::open_in_memory().expect("schema");
        let error = import(&mut target, &scratch, &path).expect_err("refused");
        assert!(error.to_string().contains("examdeck/99"), "{error}");

        std::fs::remove_file(&path).ok();
    }

    fn rewrite_manifest(path: &Path, manifest: &Manifest) {
        let original = std::fs::File::open(path).expect("open");
        let mut archive = zip::ZipArchive::new(original).expect("archive");
        let names: Vec<String> = archive.file_names().map(str::to_string).collect();
        let mut entries: Vec<(String, String)> = Vec::new();
        for name in names {
            let mut raw = String::new();
            archive
                .by_name(&name)
                .expect("entry")
                .read_to_string(&mut raw)
                .expect("read");
            entries.push((name, raw));
        }

        let rewritten = std::fs::File::create(path).expect("create");
        let mut writer = zip::ZipWriter::new(rewritten);
        let options = SimpleFileOptions::default();
        for (name, raw) in entries {
            writer.start_file(&name, options).expect("start");
            let body = if name == MANIFEST {
                serde_json::to_string_pretty(manifest).expect("manifest")
            } else {
                raw
            };
            writer.write_all(body.as_bytes()).expect("write");
        }
        writer.finish().expect("finish");
    }

    #[test]
    fn an_imported_deck_is_immediately_trainable() {
        let mut source = db::open_in_memory().expect("schema");
        let (binder_id, _) = db::insert_binder(
            &mut source,
            "AI-900",
            "AI-900",
            "x.pdf",
            "generic",
            &[question(1), question(2)],
        )
        .expect("insert");
        let scratch = std::env::temp_dir().join("oet-deck-trainable");
        let path = std::env::temp_dir().join("openexamtrainer-trainable.examdeck");
        export(&source, binder_id, &scratch, &path).expect("export");

        let mut target = db::open_in_memory().expect("schema");
        let imported = import(&mut target, &scratch, &path).expect("import");
        let session = db::session_questions(
            &target,
            imported.id,
            SessionMode::Practice,
            None,
            &db::RuleSet::default(),
        )
        .expect("set");

        assert_eq!(session.len(), 2);
        std::fs::remove_file(&path).ok();
    }
}

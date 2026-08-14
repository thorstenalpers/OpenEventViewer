//! What the user's own notes can be turned into.
//!
//! Notes are the one thing in this app nobody else wrote: the questions come from an imported dump,
//! the links from the vendor, and the notes from the person studying. Everything here takes those
//! notes and makes them usable somewhere else — a summary to reread, an episode to listen to on the
//! way somewhere — and writes the result into the project's own folder, where it can be deleted
//! again without leaving a record behind.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::assistant;
use crate::dto::Note;
use crate::error::{AppError, AppResult};

/// A file this app made out of someone's notes.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artefact {
    /// The file name, which is also how it is deleted — no path crosses the bridge.
    pub name: String,
    pub kind: String,
    pub bytes: u64,
    pub path: String,
}

fn folder(data_dir: &Path, binder_id: i64) -> AppResult<PathBuf> {
    let dir = data_dir.join("artefacts").join(binder_id.to_string());
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// What the notes of one project say, as one document.
///
/// Empty notes are refused rather than summarised: a summary of nothing is a paragraph of the
/// model's own invention, which is exactly what a study aid must not be.
pub fn collect(notes: &[Note]) -> AppResult<String> {
    let joined = notes
        .iter()
        .map(|note| note.body_md.trim())
        .filter(|body| !body.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    if joined.is_empty() {
        return Err(AppError::Message("there are no notes to work from".into()));
    }
    Ok(joined)
}

fn prompt(title: &str, notes: &str) -> String {
    format!(
        "Below are my own study notes for the {title} certification exam.\n\n\
         Rewrite them as a study summary in Markdown: a short opening paragraph, then the material \
         grouped under `##` headings, then a list of the things most likely to be asked. Keep every \
         fact that is in the notes and add none that is not. Answer with the summary alone.\n\n\
         ---\n\n{notes}"
    )
}

/// Asks the assistant to turn the notes into a summary and writes it beside the project.
pub fn summarise(
    data_dir: &Path,
    binder_id: i64,
    title: &str,
    notes: &[Note],
    source: assistant::Source,
    stamp: &str,
) -> AppResult<Artefact> {
    let summary = assistant::ask(source, &prompt(title, &collect(notes)?))?;
    let name = format!("{}-{stamp}.md", slug(title));
    let path = folder(data_dir, binder_id)?.join(&name);
    std::fs::write(&path, summary)?;
    describe(&path)
}

/// Everything made from this project's notes, newest last.
pub fn list(data_dir: &Path, binder_id: i64) -> AppResult<Vec<Artefact>> {
    let mut found: Vec<Artefact> = std::fs::read_dir(folder(data_dir, binder_id)?)?
        .flatten()
        .filter(|entry| entry.path().is_file())
        .map(|entry| describe(&entry.path()))
        .collect::<AppResult<Vec<_>>>()?;
    found.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(found)
}

/// Deletes one artefact by name.
///
/// By name and never by path: the name is checked against the folder's own listing, so a caller
/// cannot ask for a file somewhere else on the machine by dressing it up as an artefact.
pub fn delete(data_dir: &Path, binder_id: i64, name: &str) -> AppResult<Vec<Artefact>> {
    let dir = folder(data_dir, binder_id)?;
    let target = dir.join(name);
    let inside = target
        .parent()
        .is_some_and(|parent| parent == dir.as_path())
        && !name.is_empty()
        && !name.contains(['/', '\\']);

    if !inside {
        return Err(AppError::Message(format!("no artefact called {name}")));
    }
    if target.is_file() {
        std::fs::remove_file(&target)?;
    }
    list(data_dir, binder_id)
}

pub fn path_of(data_dir: &Path, binder_id: i64, name: &str) -> AppResult<PathBuf> {
    if name.is_empty() || name.contains(['/', '\\']) {
        return Err(AppError::Message(format!("no artefact called {name}")));
    }
    Ok(folder(data_dir, binder_id)?.join(name))
}

fn describe(path: &Path) -> AppResult<Artefact> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_owned();
    let kind = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase();
    Ok(Artefact {
        name,
        kind,
        bytes: std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0),
        path: path.to_string_lossy().into_owned(),
    })
}

/// A file name that survives every filesystem: letters, digits and hyphens.
fn slug(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = cleaned.trim_matches('-').to_lowercase();
    if trimmed.is_empty() {
        "notes".to_owned()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(body: &str) -> Note {
        Note {
            id: 0,
            question_id: None,
            body_md: body.to_owned(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn notes_become_one_document_separated_by_rules() {
        let joined = collect(&[note("first"), note("  "), note("second")]).expect("joined");

        assert_eq!(joined, "first\n\n---\n\nsecond");
    }

    /// Nothing in, nothing out: a summary of no notes would be the model's own invention.
    #[test]
    fn nothing_to_summarise_is_an_error_rather_than_an_empty_document() {
        let error = collect(&[note("   ")]).expect_err("no notes");

        assert!(error.to_string().contains("no notes"), "{error}");
    }

    #[test]
    fn a_title_becomes_a_file_name_anything_can_store() {
        assert_eq!(slug("AI-900 (mock)"), "ai-900--mock");
        assert_eq!(slug("???"), "notes");
    }

    /// The name comes from the webview, so it is not allowed to point anywhere but the folder.
    #[test]
    fn an_artefact_name_that_climbs_out_is_refused() {
        let data_dir = std::env::temp_dir().join("openexamtrainer-artefact-test");

        assert!(delete(&data_dir, 1, "../escaped.md").is_err());
        assert!(path_of(&data_dir, 1, "sub/dir.md").is_err());

        let _ = std::fs::remove_dir_all(&data_dir);
    }
}

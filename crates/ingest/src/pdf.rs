use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use pdfium_render::prelude::*;

use crate::layout::{self, AssemblyConfig, Glyph};
use crate::model::{Document, PageGeometry};
use crate::IngestError;

/// Binds to the Pdfium shared library.
///
/// Search order: `PDFIUM_LIB_PATH`, the directories in `extra`, the working directory, then
/// whatever the system linker resolves.
pub fn bind(extra: &[PathBuf]) -> Result<Pdfium, IngestError> {
    let mut candidates: Vec<PathBuf> = std::env::var_os("PDFIUM_LIB_PATH")
        .map(PathBuf::from)
        .into_iter()
        .collect();
    candidates.extend(extra.iter().cloned());
    candidates.push(PathBuf::from("."));

    for directory in &candidates {
        let name = Pdfium::pdfium_platform_library_name_at_path(directory);
        if let Ok(bindings) = Pdfium::bind_to_library(name) {
            return Ok(Pdfium::new(bindings));
        }
    }

    Pdfium::bind_to_system_library()
        .map(Pdfium::new)
        .map_err(|e| {
            IngestError::Pdf(format!(
                "{e}; searched {}",
                candidates
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
}

/// The Pdfium copy vendored into the repository, used by the CLI and the fixture tests.
pub fn vendored_library_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium/bin")
        .to_path_buf()
}

/// Reads a PDF through the process-wide Pdfium instance, binding it on first use.
///
/// Pdfium initialises global state and refuses a second initialisation in the same process, so the
/// instance is a singleton and every read through it is serialised.
pub fn read_file(path: &Path, search: &[PathBuf]) -> Result<Document, IngestError> {
    with_document(path, search, read)
}

/// Opens a document through the process-wide Pdfium instance and hands it to `visit`.
///
/// Every caller goes through here. Pdfium initialises global state and refuses a second
/// initialisation in the same process, so the instance is a singleton and the work is serialised —
/// binding it twice is an error, not a slowdown.
pub fn with_document<T>(
    path: &Path,
    search: &[PathBuf],
    visit: impl FnOnce(&PdfDocument<'_>) -> Result<T, IngestError>,
) -> Result<T, IngestError> {
    static INSTANCE: OnceLock<Mutex<Option<Pdfium>>> = OnceLock::new();

    let cell = INSTANCE.get_or_init(|| Mutex::new(None));
    let mut guard = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if guard.is_none() {
        *guard = Some(bind(search)?);
    }

    let pdfium = guard.as_ref().expect("bound above");
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| IngestError::Pdf(e.to_string()))?;
    visit(&document)
}

/// Reads a PDF into positioned text.
///
/// Characters are collected individually rather than as Pdfium's own text segments: the line
/// assembly in `layout` has to decide for itself where a run ends, because page furniture shares
/// baselines with body text and a segment boundary drawn by Pdfium does not know the difference.
fn read(document: &PdfDocument<'_>) -> Result<Document, IngestError> {
    let mut pages = Vec::new();
    let mut glyphs = Vec::new();

    for (index, page) in document.pages().iter().enumerate() {
        let page_index = index as u16 + 1;
        let height = page.height().value;
        pages.push(PageGeometry {
            index: page_index,
            width: page.width().value,
            height,
        });

        let text = page.text().map_err(|e| IngestError::Pdf(e.to_string()))?;
        for character in text.chars().iter() {
            let Some(ch) = character.unicode_char() else {
                continue;
            };
            if ch.is_control() && ch != '\t' {
                continue;
            }
            let Ok(bounds) = character.loose_bounds() else {
                continue;
            };
            glyphs.push(Glyph {
                page: page_index,
                x: bounds.left().value,
                y: height - bounds.top().value,
                width: (bounds.right().value - bounds.left().value).max(0.0),
                height: (bounds.top().value - bounds.bottom().value).max(0.0),
                font_size: character.scaled_font_size().value,
                ch,
            });
        }
    }

    Ok(Document {
        pages,
        lines: layout::assemble_lines(&glyphs, AssemblyConfig::default()),
    })
}

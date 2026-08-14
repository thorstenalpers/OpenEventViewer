pub mod bank;
pub mod confidence;
pub mod layout;
pub mod model;
pub mod parse;
pub mod profiles;
pub mod vce;

#[cfg(feature = "pdfium")]
pub mod figures;
#[cfg(feature = "pdfium")]
pub mod pdf;

use model::{Document, ExtractionReport};
use profiles::Profile;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("pdf backend: {0}")]
    Pdf(String),
    #[error("{0}")]
    Vce(#[from] vce::VceError),
}

/// Runs the full pipeline over an already-extracted document: furniture removal, reading order,
/// question recovery, scoring.
pub fn extract(document: &Document) -> ExtractionReport {
    let profile = Profile::detect(&document.lines);
    let (kept, furniture_dropped) = layout::strip_furniture(
        document,
        &profile.furniture,
        layout::FurnitureConfig::default(),
    );
    let ordered = layout::reading_order(&kept, &document.pages);
    let mut questions = parse::parse(&ordered, &profile);
    let missing_numbers = confidence::score(&mut questions);

    let mut stub_markers = Vec::new();
    questions.retain(|question| {
        let is_stub = question.options.is_empty() && question.answer_letters.is_empty();
        if is_stub {
            stub_markers.push(question.number);
        }
        !is_stub
    });

    ExtractionReport {
        profile: profile.id.to_string(),
        pages: document.pages.len(),
        questions,
        furniture_dropped,
        missing_numbers,
        stub_markers,
    }
}

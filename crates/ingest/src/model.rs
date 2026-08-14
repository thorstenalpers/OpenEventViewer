use serde::{Deserialize, Serialize};

/// Geometry of one source page, in PDF points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageGeometry {
    pub index: u16,
    pub width: f32,
    pub height: f32,
}

/// A horizontal run of text with its position on the page.
///
/// `y` is measured downwards from the top edge, unlike PDF's own upward axis, so that
/// sorting ascending yields reading order without a per-page flip at every call site.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine {
    pub page: u16,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub text: String,
}

impl TextLine {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

/// A source document reduced to positioned text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Document {
    pub pages: Vec<PageGeometry>,
    pub lines: Vec<TextLine>,
}

impl Document {
    pub fn page(&self, index: u16) -> Option<&PageGeometry> {
        self.pages.iter().find(|p| p.index == index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    SingleChoice,
    MultipleChoice,
    Matrix,
    ImageBased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    TextLayer,
    Ocr,
    VisionModel,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Option_ {
    pub letter: char,
    pub text: String,
    pub is_correct: bool,
}

/// Where a figure would sit: the vertical gap between the end of the stem and the first option,
/// in the coordinate space of `TextLine`. Recording the band is not the same as finding a figure —
/// `figures` stays empty until something is actually rendered out of it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FigureBand {
    pub page: u16,
    pub top: f32,
    /// `None` when the options continue overleaf: the gap then runs to the foot of `page`, whose
    /// height only the renderer knows.
    pub bottom: Option<f32>,
}

/// One box of a `Matrix` question, recovered from the `Box n:` lines of an explanation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixBox {
    pub index: u16,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    pub number: u32,
    pub topic: Option<u16>,
    pub kind: QuestionKind,
    pub stem: String,
    pub options: Vec<Option_>,
    pub answer_letters: Vec<char>,
    pub matrix: Vec<MatrixBox>,
    pub explanation: String,
    pub references: Vec<String>,
    pub source_page: u16,
    pub provenance: Provenance,
    pub confidence: f32,
    pub needs_source: bool,
    pub warnings: Vec<Warning>,
    #[serde(default)]
    pub figure_band: Option<FigureBand>,
    /// Content hashes into `ExtractionReport::assets`.
    #[serde(default)]
    pub figures: Vec<String>,
}

impl Question {
    /// How many options the user has to select. Driven by the answer key, never guessed.
    pub fn required_selections(&self) -> usize {
        self.answer_letters.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "code", content = "detail")]
pub enum Warning {
    NumberOutOfSequence { expected: u32, found: u32 },
    OptionLettersNotSequential,
    AnswerWithoutOption(char),
    MissingAnswer,
    StemTooShort,
    FigureMissing,
}

/// A rendered figure, addressed by the hash of its own bytes so the same screenshot imported twice
/// is stored once.
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    pub hash: String,
    pub png: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionReport {
    pub profile: String,
    pub pages: usize,
    pub questions: Vec<Question>,
    pub furniture_dropped: usize,
    /// Numbers the markers skipped, i.e. questions present in the source that were not recovered.
    pub missing_numbers: Vec<u32>,
    /// Markers that carried neither options nor an answer. Free dump samples end with one of
    /// these — a `NEW QUESTION n` followed by an ellipsis and an advertisement — and it is a
    /// truncation notice, not a question.
    pub stub_markers: Vec<u32>,
}

impl ExtractionReport {
    pub fn needs_review(&self) -> impl Iterator<Item = &Question> {
        self.questions.iter().filter(|q| q.confidence < 0.75)
    }
}

use openexamtrainer_ingest::model::{MatrixBox, Question, QuestionKind, Warning};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerOption {
    pub letter: char,
    pub text: String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionDto {
    #[serde(default)]
    pub id: i64,
    pub number: u32,
    pub topic: Option<u16>,
    pub kind: QuestionKind,
    pub stem: String,
    pub options: Vec<AnswerOption>,
    pub answer_letters: Vec<char>,
    pub matrix: Vec<MatrixBox>,
    pub explanation: String,
    pub references: Vec<String>,
    pub source_page: u16,
    pub confidence: f32,
    pub needs_source: bool,
    pub warnings: Vec<Warning>,
    /// Content hashes of the figures recovered for this question, in reading order.
    #[serde(default)]
    pub figures: Vec<String>,
}

impl From<&Question> for QuestionDto {
    fn from(question: &Question) -> Self {
        Self {
            id: 0,
            number: question.number,
            topic: question.topic,
            kind: question.kind,
            stem: question.stem.clone(),
            options: question
                .options
                .iter()
                .map(|option| AnswerOption {
                    letter: option.letter,
                    text: option.text.clone(),
                    is_correct: option.is_correct,
                })
                .collect(),
            answer_letters: question.answer_letters.clone(),
            matrix: question.matrix.clone(),
            explanation: question.explanation.clone(),
            references: question.references.clone(),
            source_page: question.source_page,
            confidence: question.confidence,
            needs_source: question.needs_source,
            warnings: question.warnings.clone(),
            figures: question.figures.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Binder {
    pub id: i64,
    pub title: String,
    pub certification: String,
    /// Where the vendor documents this exam, taken from the template it was created from.
    #[serde(default)]
    pub doc_url: String,
    pub source_file: String,
    pub profile: String,
    pub question_count: i64,
    pub needs_review_count: i64,
    pub needs_source_count: i64,
    pub imported_at: String,
    pub last_studied_at: Option<String>,
    pub attempt_count: i64,
    pub accuracy: Option<f64>,
    /// The catalog entry this binder was published as, so a finished challenge knows which board
    /// its result belongs on. `None` until it is published.
    pub remote_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    #[serde(default)]
    pub id: i64,
    pub question_id: Option<i64>,
    pub url: String,
    pub title: String,
    /// What the link is for, in the user's own words.
    #[serde(default)]
    pub description: String,
    /// `course`, `video`, `docs` or `other` — what waits at the other end, so a two-hour course is
    /// not offered as though it were a paragraph of documentation.
    #[serde(default)]
    pub kind: String,
    /// How long it takes, where that is known.
    #[serde(default)]
    pub minutes: Option<i64>,
}

/// An exam as it exists before anyone studies for it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    #[serde(default)]
    pub id: i64,
    pub name: String,
    pub doc_url: String,
}

/// One time this exam was passed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Certification {
    #[serde(default)]
    pub id: i64,
    /// A date, not a timestamp: nobody remembers the hour they passed.
    pub passed_at: String,
    #[serde(default)]
    pub note: String,
}

/// One exam on the overview's time axis: when it began, when it was last touched, and every time
/// it was passed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamTimeline {
    pub binder_id: i64,
    pub title: String,
    pub certification: String,
    pub started_at: String,
    pub last_studied_at: Option<String>,
    pub question_count: i64,
    pub passed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    #[serde(default)]
    pub id: i64,
    pub question_id: Option<i64>,
    pub url: String,
    pub title: String,
    pub start_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(default)]
    pub id: i64,
    pub question_id: Option<i64>,
    pub body_md: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub binder: Binder,
    pub profile: String,
    pub pages: usize,
    pub furniture_dropped: usize,
    pub missing_numbers: Vec<u32>,
    pub stub_markers: Vec<u32>,
    pub figures_recovered: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMode {
    Practice,
    Focus,
    Due,
    /// Everything ever missed that has not yet been retired — see `db::session_questions`.
    Weak,
    Exam,
    Challenge,
}

impl SessionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Practice => "practice",
            Self::Focus => "focus",
            Self::Due => "due",
            Self::Weak => "weak",
            Self::Exam => "exam",
            Self::Challenge => "challenge",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "focus" => Self::Focus,
            "due" => Self::Due,
            "weak" => Self::Weak,
            "exam" => Self::Exam,
            "challenge" => Self::Challenge,
            _ => Self::Practice,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub binder_id: i64,
    pub binder_title: String,
    pub mode: SessionMode,
    pub rules: crate::db::RuleSet,
    pub questions: Vec<QuestionDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session_id: i64,
    pub binder_id: i64,
    pub mode: SessionMode,
    pub total: i64,
    pub correct: i64,
    pub elapsed_ms: i64,
    pub wrong_question_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptResult {
    pub correct: bool,
    pub answer_letters: Vec<char>,
}

/// Who this machine publishes as. Stands in for an account: the id is drawn once and never shown,
/// the name is display text and can be changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub owner_id: String,
    pub owner_name: String,
    /// Whether this machine published it — what a server would decide, and here only asserted.
    pub mine: bool,
    pub title: String,
    pub certification: String,
    pub profile: String,
    pub question_count: i64,
    pub needs_source_count: i64,
    pub bytes: i64,
    pub published_at: String,
    pub updated_at: String,
    pub rating_count: i64,
    /// `None` until somebody rates it — no rating is not nought stars.
    pub rating: Option<f64>,
}

/// Exactly what publishing would put in the catalog, measured off the deck rather than counted off
/// the tables.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPreview {
    pub title: String,
    pub certification: String,
    pub question_count: i64,
    pub link_count: i64,
    pub video_count: i64,
    pub note_count: i64,
    pub figure_count: i64,
    pub bytes: i64,
    pub includes_source: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rating {
    pub rater_id: String,
    pub rater_name: String,
    pub mine: bool,
    pub stars: i64,
    pub comment: String,
    pub rated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardRow {
    pub runner_id: String,
    pub runner_name: String,
    pub mine: bool,
    pub seed: i64,
    pub question_count: i64,
    pub correct: i64,
    pub elapsed_ms: i64,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub pushed: i64,
    pub pulled: i64,
    pub skipped: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    /// Whether the Log entry appears in the sidebar at all.
    #[serde(default)]
    pub show_logs: bool,
    /// Whether `debug` entries are recorded. Off by default: they are the noisy ones, and a buffer
    /// full of them evicts the entries someone actually went looking for.
    #[serde(default)]
    pub debug_logging: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            show_logs: false,
            debug_logging: false,
        }
    }
}

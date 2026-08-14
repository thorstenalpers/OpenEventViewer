//! Imports a community question bank — a Markdown or JSON file rather than a vendor PDF.
//!
//! This path exists because the openly published banks are the honest alternative to exam dumps,
//! and they are all plain text. It shares `Question`, the confidence scorer and the review flow
//! with the PDF pipeline; only the recovery of the fields differs.
//!
//! No PDF backend is involved, so this module builds and tests without pdfium.

use serde::Deserialize;

use crate::confidence;
use crate::model::{ExtractionReport, Option_, Provenance, Question, QuestionKind};

#[derive(Debug, thiserror::Error)]
pub enum BankError {
    #[error("the JSON is not a list of questions: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "no questions found — expected `<h5>` headings with an `<ol>` and a `<details>` answer"
    )]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Markdown,
    Json,
}

impl Format {
    /// The format a file name implies, or `None` when this pipeline does not handle it.
    pub fn of(file_name: &str) -> Option<Self> {
        match file_name.rsplit('.').next()?.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

pub fn parse(text: &str, format: Format) -> Result<ExtractionReport, BankError> {
    let mut questions = match format {
        Format::Markdown => from_markdown(text),
        Format::Json => from_json(text)?,
    };
    if questions.is_empty() {
        return Err(BankError::Empty);
    }

    let missing_numbers = confidence::score(&mut questions);
    Ok(ExtractionReport {
        profile: match format {
            Format::Markdown => "bank-markdown".to_string(),
            Format::Json => "bank-json".to_string(),
        },
        pages: 0,
        questions,
        furniture_dropped: 0,
        missing_numbers,
        stub_markers: Vec::new(),
    })
}

/// The shape the published banks actually use: an `<h5>` heading, an `<ol>` of `<li>` options, and
/// the key hidden behind a `<details>` disclosure so the reader can try first.
///
/// Markdown equivalents (`##### `, `1. `, `Answer:`) are accepted for the same roles, because a
/// bank kept as plain Markdown is the same document without the HTML.
fn from_markdown(text: &str) -> Vec<Question> {
    let mut questions = Vec::new();
    let mut current: Option<Draft> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        if let Some(heading) = heading(trimmed) {
            if let Some(draft) = current.take() {
                questions.extend(draft.finish());
            }
            let (number, stem) = split_number(&heading);
            current = Some(Draft {
                number: number.unwrap_or(questions.len() as u32 + 1),
                stem,
                options: Vec::new(),
                answer: None,
            });
            continue;
        }

        let Some(draft) = current.as_mut() else {
            continue;
        };

        // The answer has to be tested before the option shape: `<p>` and `1. ` never collide, but a
        // bank that writes `Answer: …` as a list item would otherwise become an option.
        if let Some(answer) = answer(trimmed) {
            draft.answer = Some(answer);
        } else if draft.answer.is_none() {
            if let Some(option) = option(trimmed) {
                draft.options.push(option);
            }
        }
    }

    if let Some(draft) = current {
        questions.extend(draft.finish());
    }
    questions
}

struct Draft {
    number: u32,
    stem: String,
    options: Vec<String>,
    answer: Option<String>,
}

impl Draft {
    fn finish(self) -> Option<Question> {
        if self.stem.is_empty() || self.options.len() < 2 {
            return None;
        }
        let answers = self
            .answer
            .as_deref()
            .map(split_answers)
            .unwrap_or_default();
        Some(assemble(
            self.number,
            self.stem,
            self.options,
            answers,
            String::new(),
            Vec::new(),
        ))
    }
}

fn heading(line: &str) -> Option<String> {
    for tag in ["h5", "h4", "h3"] {
        if let Some(rest) = line.strip_prefix(&format!("<{tag}>")) {
            if let Some(inner) = rest.strip_suffix(&format!("</{tag}>")) {
                return Some(clean(inner));
            }
        }
    }
    line.strip_prefix("##### ")
        .or_else(|| line.strip_prefix("#### "))
        .map(clean)
        .filter(|heading| !heading.is_empty())
}

/// Splits `12. Which service…` into its marker and its stem. A bank that does not number its
/// questions keeps the whole heading as the stem and is numbered by position instead.
fn split_number(heading: &str) -> (Option<u32>, String) {
    let digits: String = heading.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return (None, heading.to_string());
    }
    let rest = heading[digits.len()..]
        .trim_start()
        .trim_start_matches(['.', ')'])
        .trim_start();
    match (digits.parse::<u32>(), rest.is_empty()) {
        (Ok(number), false) => (Some(number), rest.to_string()),
        _ => (None, heading.to_string()),
    }
}

fn option(line: &str) -> Option<String> {
    if let Some(rest) = line.strip_prefix("<li>") {
        let inner = rest.strip_suffix("</li>").unwrap_or(rest);
        return Some(clean(inner)).filter(|text| !text.is_empty());
    }

    let rest = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| {
            let digits: String = line.chars().take_while(char::is_ascii_digit).collect();
            (!digits.is_empty())
                .then(|| line[digits.len()..].strip_prefix(". "))
                .flatten()
        })?;
    Some(clean(rest)).filter(|text| !text.is_empty())
}

fn answer(line: &str) -> Option<String> {
    if let Some(rest) = line.strip_prefix("<p>") {
        let inner = rest.strip_suffix("</p>").unwrap_or(rest);
        return Some(clean(inner)).filter(|text| !text.is_empty());
    }
    for prefix in ["**Answer:**", "*Answer:*", "Answer:", "**Answer**:"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(clean(rest)).filter(|text| !text.is_empty());
        }
    }
    None
}

/// A multi-answer key is written as a Python list literal — `['inclusiveness', 'fairness']`.
///
/// That is not a format anyone designed; it is what falls out of the script the bank's author used.
/// Reading it is the difference between a three-of-six question arriving whole and arriving with
/// one nonsense answer whose text is the literal `['inclusiveness', …]`.
fn split_answers(answer: &str) -> Vec<String> {
    let answer = answer.trim();
    let Some(inner) = answer
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return vec![answer.to_string()];
    };

    let mut out = Vec::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\'' && c != '"' {
            continue;
        }
        let quote = c;
        let mut value = String::new();
        for c in chars.by_ref() {
            if c == quote {
                break;
            }
            value.push(c);
        }
        let value = value.trim().to_string();
        if !value.is_empty() {
            out.push(value);
        }
    }

    if out.is_empty() {
        vec![answer.to_string()]
    } else {
        out
    }
}

#[derive(Debug, Deserialize)]
struct JsonQuestion {
    #[serde(alias = "stem", alias = "text")]
    question: String,
    #[serde(alias = "choices")]
    options: Vec<String>,
    #[serde(default, alias = "answers", alias = "correct")]
    answer: JsonAnswer,
    #[serde(default)]
    explanation: String,
    #[serde(default, alias = "links")]
    references: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
enum JsonAnswer {
    #[default]
    None,
    One(String),
    Many(Vec<String>),
}

impl JsonAnswer {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::None => Vec::new(),
            Self::One(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

fn from_json(text: &str) -> Result<Vec<Question>, BankError> {
    let raw: Vec<JsonQuestion> = serde_json::from_str(text)?;
    Ok(raw
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            (item.options.len() >= 2).then(|| {
                assemble(
                    index as u32 + 1,
                    clean(&item.question),
                    item.options.iter().map(|o| clean(o)).collect(),
                    item.answer.into_vec(),
                    clean(&item.explanation),
                    item.references,
                )
            })
        })
        .collect())
}

/// Resolves the answer key against the options and builds the question.
///
/// An answer that matches no option leaves `answer_letters` empty rather than guessing. The scorer
/// then flags it and the question lands in Review — a bank with a typo in its key is a question the
/// user has to look at, not one the importer may quietly invent an answer for.
fn assemble(
    number: u32,
    stem: String,
    option_texts: Vec<String>,
    answers: Vec<String>,
    explanation: String,
    references: Vec<String>,
) -> Question {
    let letters: Vec<char> = answers
        .iter()
        .filter_map(|answer| resolve(answer, &option_texts))
        .collect();

    let options: Vec<Option_> = option_texts
        .into_iter()
        .enumerate()
        .map(|(index, text)| {
            let letter = (b'A' + index as u8) as char;
            Option_ {
                letter,
                is_correct: letters.contains(&letter),
                text,
            }
        })
        .collect();

    let kind = if letters.len() > 1 {
        QuestionKind::MultipleChoice
    } else {
        QuestionKind::SingleChoice
    };

    Question {
        number,
        topic: None,
        kind,
        stem,
        options,
        answer_letters: letters,
        matrix: Vec::new(),
        explanation,
        references,
        source_page: 0,
        provenance: Provenance::Manual,
        confidence: 1.0,
        needs_source: false,
        warnings: Vec::new(),
        figure_band: None,
        figures: Vec::new(),
    }
}

/// The option an answer names: by exact text, then by letter, then by an unambiguous prefix.
///
/// Text wins because that is what the banks write, and because an option whose own text happens to
/// be `B` would otherwise be unreachable.
///
/// The prefix pass exists for the annotated key — one bank answers `AI (.ai)` where the option
/// reads `AI`. It resolves only when **exactly one** option is involved, so it is a determination
/// and not a guess: `Azure Machine Learning` against options that also contain `Azure Machine
/// Learning Studio` matches twice and is left unanswered for the user to settle in Review.
fn resolve(answer: &str, options: &[String]) -> Option<char> {
    let letter_of = |index: usize| (b'A' + index as u8) as char;
    let wanted = normalise(answer);

    if let Some(index) = options.iter().position(|text| normalise(text) == wanted) {
        return Some(letter_of(index));
    }

    let mut chars = answer.trim().chars();
    if let Some(single) = chars.next().filter(|_| chars.next().is_none()) {
        let index = (single.to_ascii_uppercase() as usize).checked_sub(b'A' as usize)?;
        return (index < options.len()).then(|| letter_of(index));
    }

    let mut prefixed = options.iter().enumerate().filter(|(_, text)| {
        let text = normalise(text);
        text.len() >= 2
            && wanted.len() >= 2
            && (wanted.starts_with(&text) || text.starts_with(&wanted))
    });
    let (index, _) = prefixed.next()?;
    prefixed.next().is_none().then(|| letter_of(index))
}

fn normalise(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .trim_end_matches('.')
        .to_string()
}

/// Strips inline markup and decodes the entities a hand-written bank actually contains.
fn clean(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '<' => {
                for c in chars.by_ref() {
                    if c == '>' {
                        break;
                    }
                }
            }
            '&' => {
                let entity: String = chars
                    .clone()
                    .take_while(|c| *c != ';' && c.is_ascii_alphanumeric() || *c == '#')
                    .collect();
                match decode(&entity) {
                    // `Q&A Maker` is an option in one of these banks: a bare `&` is not an entity.
                    Some(decoded) => {
                        out.push(decoded);
                        for _ in 0..=entity.chars().count() {
                            chars.next();
                        }
                    }
                    None => out.push('&'),
                }
            }
            _ => out.push(c),
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "nbsp" => Some(' '),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL: &str = r#"
# Practice Questions
__Disclaimer:__ These are indicative only.

<h5>1. In this computer vision task, individual pixels are classified.</h5>
<ol type="a">
  <li>Object Detection</li>
  <li>Semantic Segmentation</li>
  <li>Image Analysis</li>
  <li>Optical Character Recognition</li>
</ol>
<details>
  <summary>Show Answer</summary>
  <p>Semantic Segmentation</p>
</details>

<h5>2. What are three Microsoft guiding principles for responsible AI?</h5>
<ol type='a'>
  <li>knowledgeability</li>
  <li>decisiveness</li>
  <li>inclusiveness</li>
  <li>fairness</li>
  <li>opinionatedness</li>
  <li>reliability and safety</li>
</ol>
<details>
  <summary>Show Answer</summary>
  <p>['inclusiveness', 'fairness', 'reliability and safety']</p>
</details>
"#;

    #[test]
    fn the_published_bank_shape_is_read_whole() {
        let report = parse(REAL, Format::Markdown).expect("parses");

        assert_eq!(report.questions.len(), 2);

        let first = &report.questions[0];
        assert_eq!(first.number, 1);
        assert_eq!(
            first.stem,
            "In this computer vision task, individual pixels are classified."
        );
        assert_eq!(first.kind, QuestionKind::SingleChoice);
        assert_eq!(first.options.len(), 4);
        assert_eq!(first.options[1].text, "Semantic Segmentation");
        assert_eq!(first.answer_letters, vec!['B']);
        assert!(first.options[1].is_correct);
        assert_eq!(first.confidence, 1.0);
        assert!(first.warnings.is_empty());
    }

    /// The prose around the questions is not a question. Without the `<h5>` gate, the disclaimer's
    /// own list items would arrive as options on whatever came before them.
    #[test]
    fn surrounding_prose_is_not_mistaken_for_content() {
        let report = parse(REAL, Format::Markdown).expect("parses");

        assert!(report
            .questions
            .iter()
            .all(|q| !q.stem.contains("Disclaimer")));
        assert!(report
            .questions
            .iter()
            .flat_map(|q| &q.options)
            .all(|o| !o.text.contains("indicative")));
    }

    #[test]
    fn a_python_list_literal_becomes_three_correct_options() {
        let report = parse(REAL, Format::Markdown).expect("parses");
        let multi = &report.questions[1];

        assert_eq!(multi.kind, QuestionKind::MultipleChoice);
        assert_eq!(multi.answer_letters, vec!['C', 'D', 'F']);
        assert_eq!(multi.required_selections(), 3);
        assert!(multi.options.iter().filter(|o| o.is_correct).count() == 3);
        assert!(
            multi.options.iter().all(|o| !o.text.starts_with('[')),
            "the literal is a key, never an option"
        );
    }

    /// Taken verbatim from a published bank: the key annotates the option rather than repeating it.
    #[test]
    fn an_annotated_key_resolves_when_exactly_one_option_can_be_meant() {
        let source = "<h5>10. Which is not a supported image format?</h5>\n<li>JPEG</li>\n\
                      <li>AI</li>\n<li>PNG</li>\n<li>BMP</li>\n<p>AI (.ai)</p>";

        let report = parse(source, Format::Markdown).expect("parses");

        assert_eq!(report.questions[0].answer_letters, vec!['B']);
    }

    /// …and refuses when more than one could be, because picking is guessing.
    #[test]
    fn an_ambiguous_prefix_is_left_for_the_user() {
        let source = "<h5>1. Which service?</h5>\n<li>Azure Machine Learning</li>\n\
                      <li>Azure Machine Learning Studio</li>\n<p>Azure Machine</p>";

        let report = parse(source, Format::Markdown).expect("parses");

        assert!(report.questions[0].answer_letters.is_empty());
        assert!(report.questions[0].confidence < confidence::REVIEW_THRESHOLD);
    }

    /// A key that names nothing is the bank's error, and it has to survive as a question the user
    /// is asked to look at — not as a question with an invented answer.
    #[test]
    fn an_answer_that_matches_no_option_is_flagged_rather_than_guessed() {
        let source = "<h5>1. Which service?</h5>\n<li>Azure Machine Learning</li>\n<li>Azure Bot \
                      Service</li>\n<p>Azure Cognitive Search</p>";

        let report = parse(source, Format::Markdown).expect("parses");
        let question = &report.questions[0];

        assert!(question.answer_letters.is_empty());
        assert!(question.options.iter().all(|o| !o.is_correct));
        assert!(question.confidence < confidence::REVIEW_THRESHOLD);
        assert!(question
            .warnings
            .contains(&crate::model::Warning::MissingAnswer));
    }

    #[test]
    fn markdown_without_the_html_is_read_the_same_way() {
        let source = "##### 7. Which two are Azure services?\n1. Azure Machine Learning\n2. \
                      Photoshop\n3. Azure Bot Service\n\n**Answer:** ['Azure Machine Learning', \
                      'Azure Bot Service']";

        let report = parse(source, Format::Markdown).expect("parses");
        let question = &report.questions[0];

        assert_eq!(question.number, 7);
        assert_eq!(question.stem, "Which two are Azure services?");
        assert_eq!(question.options.len(), 3);
        assert_eq!(question.answer_letters, vec!['A', 'C']);
    }

    #[test]
    fn entities_are_decoded_and_a_bare_ampersand_survives() {
        let source = "<h5>1. Which tool builds a bot?</h5>\n<li>Q&A Maker</li>\n<li>&quot;Azure \
                      Bot&quot; &amp; Framework</li>\n<p>Q&A Maker</p>";

        let report = parse(source, Format::Markdown).expect("parses");
        let question = &report.questions[0];

        assert_eq!(question.options[0].text, "Q&A Maker");
        assert_eq!(question.options[1].text, "\"Azure Bot\" & Framework");
        assert_eq!(question.answer_letters, vec!['A']);
    }

    #[test]
    fn a_json_bank_accepts_a_letter_or_the_answer_text() {
        let source = r#"[
            {"question": "Which service trains models?",
             "options": ["Azure Machine Learning", "Azure Bot Service"],
             "answer": "A"},
            {"stem": "Which two are vision tasks?",
             "choices": ["OCR", "Sentiment analysis", "Object detection"],
             "answers": ["OCR", "Object detection"],
             "explanation": "Both read images.",
             "links": ["https://learn.microsoft.com/"]}
        ]"#;

        let report = parse(source, Format::Json).expect("parses");

        assert_eq!(report.profile, "bank-json");
        assert_eq!(report.questions[0].answer_letters, vec!['A']);
        assert_eq!(report.questions[1].answer_letters, vec!['A', 'C']);
        assert_eq!(report.questions[1].explanation, "Both read images.");
        assert_eq!(report.questions[1].references.len(), 1);
    }

    /// An option whose own text is a single letter must stay reachable — text is matched first.
    #[test]
    fn an_option_literally_named_b_is_still_selectable() {
        let source = r#"[{"question": "Which label?", "options": ["B", "C"], "answer": "B"}]"#;

        let report = parse(source, Format::Json).expect("parses");

        assert_eq!(report.questions[0].answer_letters, vec!['A']);
        assert!(report.questions[0].options[0].is_correct);
    }

    #[test]
    fn a_file_with_no_questions_is_refused_by_name_rather_than_imported_empty() {
        let error = parse("# Just a readme\n\nNothing here.", Format::Markdown).expect_err("empty");
        assert!(error.to_string().contains("no questions found"), "{error}");

        assert!(matches!(
            parse("{\"not\": \"a list\"}", Format::Json),
            Err(BankError::Json(_))
        ));
    }

    #[test]
    fn the_format_comes_from_the_file_name() {
        assert_eq!(Format::of("bank.md"), Some(Format::Markdown));
        assert_eq!(Format::of("README.MARKDOWN"), Some(Format::Markdown));
        assert_eq!(Format::of("questions.json"), Some(Format::Json));
        assert_eq!(Format::of("dump.pdf"), None);
        assert_eq!(Format::of("noextension"), None);
    }
}

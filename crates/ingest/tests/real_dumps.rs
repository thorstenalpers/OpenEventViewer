#![cfg(feature = "pdfium")]

use std::path::{Path, PathBuf};

use openexamtrainer_ingest::model::{
    Document, ExtractionReport, FigureBand, QuestionKind, Warning,
};
use openexamtrainer_ingest::{extract, figures, pdf, vce};

/// The fixtures are vendor exam material and are not redistributed with the source, so they are
/// gitignored. Absent fixtures skip the test loudly rather than passing on nothing.
fn fixture(name: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    if path.exists() {
        Some(path)
    } else {
        eprintln!("SKIPPED: fixture {name} is missing — see .agents/docs/03-ingest-pipeline.md");
        None
    }
}

fn read(name: &str) -> Option<(PathBuf, Document, ExtractionReport)> {
    let path = fixture(name)?;
    let document = pdf::read_file(&path, &[pdf::vendored_library_dir()]).expect("document reads");
    let report = extract(&document);
    Some((path, document, report))
}

fn report(name: &str) -> Option<ExtractionReport> {
    read(name).map(|(_, _, report)| report)
}

fn assert_no_furniture_leaked(report: &ExtractionReport, vendor: &str) {
    for question in &report.questions {
        let haystack = format!(
            "{}\n{}\n{}",
            question.stem,
            question
                .options
                .iter()
                .map(|o| o.text.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            question.explanation
        );
        assert!(
            !haystack.to_lowercase().contains(vendor),
            "question #{} carries page furniture: {haystack}",
            question.number
        );
    }
}

#[test]
fn the_certshared_sample_extracts_its_eight_real_questions() {
    let Some(report) = report("certshared-ai900.pdf") else {
        return;
    };

    assert_eq!(report.profile, "certshared");
    assert_eq!(report.pages, 6);
    assert_eq!(report.questions.len(), 8);

    // The source jumps from marker 8 to marker 10, and 10 is the paywall notice, not a question.
    assert_eq!(report.missing_numbers, vec![9]);
    assert_eq!(report.stub_markers, vec![10]);

    assert_no_furniture_leaked(&report, "certshared");

    let first = &report.questions[0];
    assert_eq!(first.number, 1);
    assert_eq!(first.topic, Some(1));
    assert_eq!(first.kind, QuestionKind::SingleChoice);
    assert_eq!(
        first.stem,
        "You are building an AI system.\nWhich task should you include to ensure that the service \
         meets the Microsoft transparency principle for responsible AI?"
    );
    assert_eq!(first.options.len(), 4);
    assert_eq!(
        first.options[0].text,
        "Ensure that all visuals have an associated text that can be read by a screen reader."
    );
    assert_eq!(first.answer_letters, vec!['C']);
    assert!(first.options[2].is_correct);
    assert_eq!(first.confidence, 1.0);
    assert_eq!(first.references.len(), 1);

    let multi = report
        .questions
        .iter()
        .find(|q| q.number == 6)
        .expect("question 6");
    assert_eq!(multi.kind, QuestionKind::MultipleChoice);
    assert_eq!(multi.answer_letters, vec!['A', 'D']);
    assert_eq!(multi.required_selections(), 2);

    let without_figure: Vec<u32> = report
        .questions
        .iter()
        .filter(|q| q.needs_source)
        .map(|q| q.number)
        .collect();
    assert_eq!(without_figure, vec![2, 3, 4, 5, 8]);
    assert!(report
        .questions
        .iter()
        .filter(|q| q.needs_source)
        .all(|q| q.confidence < 0.75));

    let matrix = report
        .questions
        .iter()
        .find(|q| q.number == 3)
        .expect("question 3");
    assert_eq!(matrix.kind, QuestionKind::Matrix);
    assert_eq!(matrix.matrix.len(), 3);
    assert_eq!(matrix.matrix[0].value, "No");
    assert_eq!(matrix.matrix[1].value, "Yes");
}

#[test]
fn the_certleader_sample_extracts_its_eleven_real_questions() {
    let Some(report) = report("certleader-ai900.pdf") else {
        return;
    };

    assert_eq!(report.profile, "certleader");
    assert_eq!(report.pages, 7);
    assert_eq!(report.questions.len(), 11);
    assert_eq!(report.stub_markers, vec![15]);

    assert_no_furniture_leaked(&report, "certleader");

    // The source itself numbers two consecutive questions 10 and then jumps to 15.
    let numbers: Vec<u32> = report.questions.iter().map(|q| q.number).collect();
    assert_eq!(numbers, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10]);

    let three_of_six = report
        .questions
        .iter()
        .find(|q| q.number == 8)
        .expect("question 8");
    assert_eq!(three_of_six.options.len(), 6);
    assert_eq!(three_of_six.answer_letters, vec!['C', 'D', 'F']);
    assert_eq!(three_of_six.required_selections(), 3);

    assert_eq!(
        report.questions.iter().filter(|q| q.needs_source).count(),
        5
    );
}

/// The `A. Mastered / B. Not Mastered` placeholder is not an empty question: the real answer area
/// is drawn in the gap above the options, as an image with no text layer of its own. Recovering it
/// turns five unanswerable questions in this file into five answerable ones.
#[test]
fn the_placeholder_questions_get_their_answer_area_back() {
    let Some((path, document, mut report)) = read("certshared-ai900.pdf") else {
        return;
    };

    let flagged: Vec<u32> = report
        .questions
        .iter()
        .filter(|q| q.needs_source)
        .map(|q| q.number)
        .collect();
    assert_eq!(flagged, vec![2, 3, 4, 5, 8]);

    let assets = figures::capture(&path, &document, &mut report).expect("capture");

    assert_eq!(assets.len(), 5, "one figure per placeholder question");
    assert!(assets.iter().all(|a| &a.png[1..4] == b"PNG"));
    assert!(assets.iter().all(|a| a.hash.len() == 64), "sha-256, hex");

    let with_figure: Vec<u32> = report
        .questions
        .iter()
        .filter(|q| !q.figures.is_empty())
        .map(|q| q.number)
        .collect();
    assert_eq!(with_figure, flagged);

    // The whole point: with the picture back they are answerable, so nothing is still flagged and
    // the confidence penalty for a missing figure is gone.
    assert!(report.questions.iter().all(|q| !q.needs_source));
    let recovered = report
        .questions
        .iter()
        .find(|q| q.number == 3)
        .expect("question 3");
    assert!(recovered.confidence >= 0.75, "{}", recovered.confidence);
    assert!(recovered
        .warnings
        .iter()
        .all(|w| !matches!(w, Warning::FigureMissing)));
}

/// A question that was never a placeholder has no hole to look in, so it gains nothing.
#[test]
fn an_ordinary_question_is_left_alone() {
    let Some((path, document, mut report)) = read("certshared-ai900.pdf") else {
        return;
    };

    figures::capture(&path, &document, &mut report).expect("capture");

    let plain = report
        .questions
        .iter()
        .find(|q| q.number == 1)
        .expect("question 1");
    assert!(plain.figures.is_empty());
    assert_eq!(plain.confidence, 1.0);
}

/// The other half of the same claim: pointed at a region that does carry ink, the same code path
/// produces a real PNG. Without this, "no assets" above would also pass on a broken rasteriser.
///
/// The mask list is emptied to make the point precisely: the rasteriser sees the ink either way,
/// and it is masking the known text lines — not a failure to render — that keeps a text-only band
/// from being mistaken for a diagram.
#[test]
fn a_band_that_carries_ink_is_captured_as_a_png() {
    let Some((path, document, mut report)) = read("certshared-ai900.pdf") else {
        return;
    };
    for question in &mut report.questions {
        question.figure_band = None;
    }

    let unmasked = Document {
        pages: document.pages.clone(),
        lines: Vec::new(),
    };

    // Aim a band at a page region that is certainly full of type.
    let first = &mut report.questions[0];
    first.figure_band = Some(FigureBand {
        page: first.source_page,
        top: 100.0,
        bottom: Some(400.0),
    });

    let assets = figures::capture(&path, &unmasked, &mut report).expect("capture");

    assert_eq!(assets.len(), 1);
    assert_eq!(report.questions[0].figures, vec![assets[0].hash.clone()]);
    assert_eq!(assets[0].hash.len(), 64, "sha-256, hex");
    assert_eq!(&assets[0].png[1..4], b"PNG");
    assert!(assets[0].png.len() > 1_000, "{} bytes", assets[0].png.len());
}

#[test]
fn the_avanset_vce_fails_with_the_pdf_route_rather_than_a_parse_error() {
    let Some(path) = fixture("avanset-az900.vce") else {
        return;
    };

    let error = vce::import(&path).expect_err("no decoder ships yet");
    let message = error.to_string();

    assert!(message.contains("85 a8 06 02 04 00 00 00"), "{message}");
    assert!(message.contains("Export the exam to PDF"), "{message}");
}

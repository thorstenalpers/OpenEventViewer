use crate::model::{Question, Warning};

pub const REVIEW_THRESHOLD: f32 = 0.75;

const NUMBER_OUT_OF_SEQUENCE: f32 = 0.30;
const OPTIONS_NOT_SEQUENTIAL: f32 = 0.25;
const ANSWER_WITHOUT_OPTION: f32 = 0.40;
const MISSING_ANSWER: f32 = 0.50;
const STEM_TOO_SHORT: f32 = 0.30;
const FIGURE_MISSING: f32 = 0.35;

const MIN_STEM_LENGTH: usize = 20;

/// Scores every question and records the marker numbers the source contained but the parse did not
/// recover. A silent shortfall would be indistinguishable from a dump that simply had fewer
/// questions, so the gap is reported rather than inferred.
pub fn score(questions: &mut [Question]) -> Vec<u32> {
    let mut missing = Vec::new();
    let mut expected: Option<u32> = None;

    for question in questions.iter_mut() {
        let mut penalty = 0.0;
        let mut warnings = Vec::new();

        if let Some(expected_number) = expected {
            if question.number != expected_number {
                penalty += NUMBER_OUT_OF_SEQUENCE;
                warnings.push(Warning::NumberOutOfSequence {
                    expected: expected_number,
                    found: question.number,
                });
                missing.extend(expected_number..question.number);
            }
        }
        expected = Some(question.number + 1);

        let sequential = question
            .options
            .iter()
            .enumerate()
            .all(|(index, option)| option.letter as u8 == b'A' + index as u8);
        if !question.options.is_empty() && !sequential {
            penalty += OPTIONS_NOT_SEQUENTIAL;
            warnings.push(Warning::OptionLettersNotSequential);
        }

        if question.answer_letters.is_empty() {
            penalty += MISSING_ANSWER;
            warnings.push(Warning::MissingAnswer);
        }
        for letter in &question.answer_letters {
            if !question.options.iter().any(|o| o.letter == *letter) {
                penalty += ANSWER_WITHOUT_OPTION;
                warnings.push(Warning::AnswerWithoutOption(*letter));
            }
        }

        if question.stem.trim().chars().count() < MIN_STEM_LENGTH {
            penalty += STEM_TOO_SHORT;
            warnings.push(Warning::StemTooShort);
        }

        if question.needs_source {
            penalty += FIGURE_MISSING;
            warnings.push(Warning::FigureMissing);
        }

        question.confidence = (1.0 - penalty).clamp(0.0, 1.0);
        question.warnings = warnings;
    }

    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Option_, Provenance, QuestionKind};

    fn question(number: u32, stem: &str, letters: &[char], answer: &[char]) -> Question {
        Question {
            number,
            topic: None,
            kind: QuestionKind::SingleChoice,
            stem: stem.to_string(),
            options: letters
                .iter()
                .map(|letter| Option_ {
                    letter: *letter,
                    text: "text".to_string(),
                    is_correct: answer.contains(letter),
                })
                .collect(),
            answer_letters: answer.to_vec(),
            matrix: Vec::new(),
            explanation: String::new(),
            references: Vec::new(),
            source_page: 1,
            provenance: Provenance::TextLayer,
            confidence: 1.0,
            needs_source: false,
            warnings: Vec::new(),
            figure_band: None,
            figures: Vec::new(),
        }
    }

    #[test]
    fn a_clean_question_scores_full_confidence() {
        let mut questions = vec![question(
            1,
            "A stem that is comfortably long enough.",
            &['A', 'B'],
            &['A'],
        )];

        let missing = score(&mut questions);

        assert!(missing.is_empty());
        assert_eq!(questions[0].confidence, 1.0);
        assert!(questions[0].warnings.is_empty());
    }

    #[test]
    fn a_skipped_marker_is_reported_as_a_missing_number() {
        let mut questions = vec![
            question(1, "A stem that is comfortably long enough.", &['A'], &['A']),
            question(4, "Another stem that is long enough here.", &['A'], &['A']),
        ];

        let missing = score(&mut questions);

        assert_eq!(missing, vec![2, 3]);
        assert!(questions[1].confidence < REVIEW_THRESHOLD);
    }

    #[test]
    fn an_answer_with_no_matching_option_is_the_heaviest_single_penalty() {
        let mut questions = vec![question(
            1,
            "A stem that is comfortably long enough.",
            &['A', 'B'],
            &['D'],
        )];

        score(&mut questions);

        assert!((questions[0].confidence - 0.6).abs() < f32::EPSILON);
        assert!(matches!(
            questions[0].warnings[0],
            crate::model::Warning::AnswerWithoutOption('D')
        ));
    }

    #[test]
    fn a_question_missing_its_figure_lands_in_review() {
        let mut questions = vec![question(
            1,
            "For each of the following statements, select Yes.",
            &['A', 'B'],
            &['A'],
        )];
        questions[0].needs_source = true;

        score(&mut questions);

        assert!(questions[0].confidence < REVIEW_THRESHOLD);
    }
}

use regex::Regex;

use crate::model::{FigureBand, MatrixBox, Option_, Provenance, Question, QuestionKind, TextLine};
use crate::profiles::Profile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Head,
    Stem,
    Options,
    Rationale,
}

struct Builder<'a> {
    number: u32,
    page: u16,
    topic: Option<u16>,
    state: State,
    stem: Vec<&'a TextLine>,
    options: Vec<(char, Vec<&'a TextLine>)>,
    answer_letters: Vec<char>,
    rationale: Vec<&'a TextLine>,
}

impl<'a> Builder<'a> {
    fn new(number: u32, page: u16) -> Self {
        Self {
            number,
            page,
            topic: None,
            state: State::Head,
            stem: Vec::new(),
            options: Vec::new(),
            answer_letters: Vec::new(),
            rationale: Vec::new(),
        }
    }

    fn next_option_letter(&self) -> char {
        match self.options.last() {
            None => 'A',
            Some((letter, _)) => ((*letter as u8) + 1) as char,
        }
    }
}

pub fn parse(lines: &[TextLine], profile: &Profile) -> Vec<Question> {
    let mut questions = Vec::new();
    let mut builder: Option<Builder> = None;

    for line in lines {
        if let Some(captures) = profile.marker.captures(&line.text) {
            if let Some(previous) = builder.take() {
                questions.push(finish(previous));
            }
            let number = captures[1].parse().unwrap_or(0);
            builder = Some(Builder::new(number, line.page));
            continue;
        }

        let Some(current) = builder.as_mut() else {
            continue;
        };

        if current.state == State::Head {
            if let Some(captures) = profile.topic.captures(&line.text) {
                current.topic = captures[1].parse().ok();
                continue;
            }
            current.state = State::Stem;
        }

        if current.state != State::Rationale {
            if let Some(captures) = profile.answer.captures(&line.text) {
                current.answer_letters = captures[1]
                    .chars()
                    .filter(|c| c.is_ascii_uppercase())
                    .collect();
                current.state = State::Rationale;
                continue;
            }
        }

        match current.state {
            State::Head => unreachable!("promoted to Stem above"),
            State::Stem | State::Options => {
                if let Some(captures) = profile.option.captures(&line.text) {
                    let letter = captures[1].chars().next().expect("group is one char");
                    if letter == current.next_option_letter() {
                        current.options.push((letter, Vec::new()));
                        current.state = State::Options;
                        current
                            .options
                            .last_mut()
                            .expect("just pushed")
                            .1
                            .push(line);
                        continue;
                    }
                }
                match current.state {
                    State::Options => current
                        .options
                        .last_mut()
                        .expect("state implies one option")
                        .1
                        .push(line),
                    _ => current.stem.push(line),
                }
            }
            State::Rationale => current.rationale.push(line),
        }
    }

    if let Some(last) = builder.take() {
        questions.push(finish(last));
    }
    questions
}

fn finish(builder: Builder<'_>) -> Question {
    let stem = join_wrapped(&builder.stem);

    let options: Vec<Option_> = builder
        .options
        .iter()
        .map(|(letter, lines)| Option_ {
            letter: *letter,
            text: strip_option_prefix(*letter, &join_wrapped(lines)),
            is_correct: builder.answer_letters.contains(letter),
        })
        .collect();

    let rationale_lines: Vec<String> = builder
        .rationale
        .iter()
        .map(|l| l.text.trim().to_string())
        .collect();
    let matrix = matrix_boxes(&rationale_lines);
    let references = collect_references(&rationale_lines);
    let explanation = join_wrapped(&builder.rationale);

    let figure_band = figure_band(&builder);
    let placeholder = is_placeholder(&options);
    let kind = if !matrix.is_empty() {
        QuestionKind::Matrix
    } else if placeholder {
        QuestionKind::ImageBased
    } else if builder.answer_letters.len() > 1 {
        QuestionKind::MultipleChoice
    } else {
        QuestionKind::SingleChoice
    };

    Question {
        number: builder.number,
        topic: builder.topic,
        kind,
        stem,
        options,
        answer_letters: builder.answer_letters,
        matrix,
        explanation,
        references,
        source_page: builder.page,
        provenance: Provenance::TextLayer,
        confidence: 1.0,
        needs_source: placeholder,
        warnings: Vec::new(),
        figure_band,
        figures: Vec::new(),
    }
}

/// The vertical gap between the last line of the stem and the first option.
///
/// A figure is the only thing that puts a hole in a question, so the hole is where to look for one.
/// Anything under a line and a half of leading is ordinary paragraph spacing.
///
/// When the options continue overleaf, the figure is what pushed them there: a diagram that no
/// longer fits is moved to the next page whole, so the gap to search is the head of the option's
/// page, not the tail of the stem's. Everything above the first option there is either the figure
/// or the running header, and the header is masked before the band is measured.
fn figure_band(builder: &Builder<'_>) -> Option<FigureBand> {
    let last_stem = builder.stem.last()?;
    let first_option = builder.options.first()?.1.first()?;
    let top = last_stem.y + last_stem.height;

    if last_stem.page != first_option.page {
        return Some(FigureBand {
            page: first_option.page,
            top: 0.0,
            bottom: Some(first_option.y),
        });
    }

    let bottom = first_option.y;
    let leading = last_stem.font_size.max(1.0) * 1.5;
    (bottom - top > leading).then_some(FigureBand {
        page: last_stem.page,
        top,
        bottom: Some(bottom),
    })
}

/// Rejoins hard-wrapped text.
///
/// A line that reaches the block's right edge was wrapped by the typesetter and continues on the
/// next line; a short line ended deliberately. Only the coordinates can tell the two apart, which
/// is why the wrap decision lives here and not in a regex over flat text.
fn join_wrapped(lines: &[&TextLine]) -> String {
    let Some(max_right) = lines
        .iter()
        .map(|l| l.right())
        .fold(None::<f32>, |acc, r| Some(acc.map_or(r, |a: f32| a.max(r))))
    else {
        return String::new();
    };
    let tolerance = (max_right * 0.03).max(4.0);

    let mut out = String::new();
    for (index, line) in lines.iter().enumerate() {
        let text = line.text.trim();
        if text.is_empty() {
            continue;
        }
        if index > 0 {
            let previous = lines[index - 1];
            if previous.right() >= max_right - tolerance {
                out.push(' ');
            } else {
                out.push('\n');
            }
        }
        out.push_str(text);
    }
    out.trim().to_string()
}

fn strip_option_prefix(letter: char, text: &str) -> String {
    let prefix_len = text
        .char_indices()
        .nth(2)
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    let (head, tail) = text.split_at(prefix_len);
    let matches_prefix = head
        .chars()
        .next()
        .is_some_and(|c| c == letter && head.chars().nth(1).is_some_and(|d| d == '.' || d == ')'));
    if matches_prefix {
        tail.trim_start().to_string()
    } else {
        text.to_string()
    }
}

fn is_placeholder(options: &[Option_]) -> bool {
    options.len() == 2
        && options[0].text.trim().eq_ignore_ascii_case("mastered")
        && options[1].text.trim().eq_ignore_ascii_case("not mastered")
}

fn matrix_boxes(lines: &[String]) -> Vec<MatrixBox> {
    let pattern = Regex::new(r"(?i)^box\s+(\d+)\s*:\s*(.+)$").expect("valid");
    let mut boxes: Vec<MatrixBox> = lines
        .iter()
        .filter_map(|line| pattern.captures(line))
        .filter_map(|captures| {
            Some(MatrixBox {
                index: captures[1].parse().ok()?,
                value: captures[2].trim().to_string(),
            })
        })
        .collect();
    boxes.sort_by_key(|b| b.index);
    boxes.dedup_by_key(|b| b.index);
    boxes
}

fn collect_references(lines: &[String]) -> Vec<String> {
    let pattern = Regex::new(r"https?://[^\s<>\)]+").expect("valid");
    let mut urls: Vec<String> = lines
        .iter()
        .flat_map(|line| {
            pattern
                .find_iter(line)
                .map(|m| m.as_str().trim_end_matches(['.', ',']).to_string())
        })
        .collect();
    urls.dedup();
    urls
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PageGeometry;

    const WIDE: f32 = 500.0;

    fn doc_lines(source: &str) -> Vec<TextLine> {
        source
            .lines()
            .enumerate()
            .map(|(index, text)| {
                let trimmed = text.trim();
                TextLine {
                    page: 1,
                    x: 50.0,
                    y: 50.0 + index as f32 * 12.0,
                    // Long lines are treated as wrapped, short ones as deliberate breaks, so the
                    // fixture has to carry a plausible width for `join_wrapped` to read.
                    width: if trimmed.len() > 90 {
                        WIDE
                    } else {
                        trimmed.len() as f32 * 5.0
                    },
                    height: 10.0,
                    font_size: 10.0,
                    text: trimmed.to_string(),
                }
            })
            .filter(|l| !l.text.is_empty())
            .collect()
    }

    fn _page() -> PageGeometry {
        PageGeometry {
            index: 1,
            width: 600.0,
            height: 800.0,
        }
    }

    #[test]
    fn a_single_choice_question_is_recovered_whole() {
        let lines = doc_lines(
            r#"
NEW QUESTION 2
- (Exam Topic 1)
You are building an AI system.
Which task should you include to ensure that the service meets the Microsoft transparency principle for responsible AI?
A. Ensure that all visuals have an associated text that can be read by a screen reader.
B. Enable autoscaling to ensure that a service scales based on demand.
C. Provide documentation to help developers debug code.
D. Ensure that a training dataset is representative of the population.
Answer: C
Explanation:
Reference:
https://docs.microsoft.com/en-us/learn/modules/responsible-ai-principles/4-guiding-principles
"#,
        );

        let questions = parse(&lines, &Profile::generic());

        assert_eq!(questions.len(), 1);
        let question = &questions[0];
        assert_eq!(question.number, 2);
        assert_eq!(question.topic, Some(1));
        assert_eq!(question.kind, QuestionKind::SingleChoice);
        assert!(question.stem.starts_with("You are building an AI system."));
        assert_eq!(question.options.len(), 4);
        assert_eq!(
            question.options[2].text,
            "Provide documentation to help developers debug code."
        );
        assert!(question.options[2].is_correct);
        assert!(!question.options[0].is_correct);
        assert_eq!(question.answer_letters, vec!['C']);
        assert_eq!(question.references.len(), 1);
        assert!(!question.needs_source);
    }

    #[test]
    fn a_multi_letter_answer_sets_the_selection_count() {
        let lines = doc_lines(
            r#"
NEW QUESTION 9
Which two services should you use? Each correct answer presents part of the solution.
A. Form Recognizer
B. Custom Vision
C. Text Analytics
D. Computer Vision
Answer: AD
"#,
        );

        let questions = parse(&lines, &Profile::generic());

        assert_eq!(questions[0].kind, QuestionKind::MultipleChoice);
        assert_eq!(questions[0].required_selections(), 2);
        assert_eq!(questions[0].answer_letters, vec!['A', 'D']);
        assert!(questions[0].options[0].is_correct);
        assert!(questions[0].options[3].is_correct);
        assert!(!questions[0].options[1].is_correct);
    }

    #[test]
    fn the_mastered_placeholder_is_a_matrix_question_missing_its_figure() {
        let lines = doc_lines(
            r#"
NEW QUESTION 3
- (Exam Topic 1)
For each of the following statements, select Yes if the statement is true. Otherwise, select No.
A. Mastered
B. Not Mastered
Answer: A
Explanation:
Box 1: No
Box 2: Yes
Box 3: Yes
"#,
        );

        let questions = parse(&lines, &Profile::generic());

        let question = &questions[0];
        assert_eq!(question.kind, QuestionKind::Matrix);
        assert!(question.needs_source);
        assert_eq!(question.matrix.len(), 3);
        assert_eq!(question.matrix[0].value, "No");
        assert_eq!(question.matrix[2].index, 3);
    }

    #[test]
    fn a_letter_out_of_sequence_is_body_text_not_an_option() {
        let lines = doc_lines(
            r#"
NEW QUESTION 5
Pick one.
A. first
B. second
D. this line is a continuation, not an option
Answer: A
"#,
        );

        let questions = parse(&lines, &Profile::generic());

        assert_eq!(questions[0].options.len(), 2);
        assert!(questions[0].options[1].text.contains("continuation"));
    }

    #[test]
    fn consecutive_questions_are_split_at_the_marker() {
        let lines = doc_lines(
            r#"
NEW QUESTION 1
First stem.
A. a
B. b
Answer: A
NEW QUESTION 2
Second stem.
A. c
B. d
Answer: B
"#,
        );

        let questions = parse(&lines, &Profile::generic());

        assert_eq!(questions.len(), 2);
        assert_eq!(questions[0].stem, "First stem.");
        assert_eq!(questions[1].stem, "Second stem.");
        assert_eq!(questions[1].answer_letters, vec!['B']);
    }
}

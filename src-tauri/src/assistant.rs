use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::dto::QuestionDto;
use crate::error::{AppError, AppResult};

const KEYRING_SERVICE: &str = "com.thorstenalpers.openexamtrainer";
const KEYRING_ACCOUNT: &str = "anthropic-api-key";

const MODEL: &str = "claude-opus-5";
const MAX_TOKENS: u32 = 8192;
const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
/// Server-side fallbacks: when a safety classifier declines the request, the API re-runs it on a
/// substitute model instead of handing back an empty response.
const BETA_FALLBACK: &str = "server-side-fallback-2026-07-01";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The local `claude` binary. Nothing leaves the machine that the binary does not already send.
    #[default]
    Cli,
    /// api.anthropic.com, with the key in the Windows Credential Manager.
    Anthropic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    /// Why the marked answer is right and the tempting one is wrong.
    Explain,
    /// Variants of a question that keeps being missed, so the concept is learned, not the wording.
    Variants,
    /// The explanation turned into a short study note.
    Note,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub source: Source,
    pub cli_available: bool,
    pub has_key: bool,
}

pub fn status(source: Source) -> Status {
    Status {
        source,
        cli_available: locate_cli().is_some(),
        has_key: entry().and_then(|e| e.get_password()).is_ok(),
    }
}

fn entry() -> keyring::Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
}

/// Stores the key where this app cannot read it back into the UI — only send it.
pub fn set_key(key: &str) -> AppResult<()> {
    let entry = entry().map_err(|error| AppError::Message(error.to_string()))?;
    if key.trim().is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry
        .set_password(key.trim())
        .map_err(|error| AppError::Message(error.to_string()))
}

const SYSTEM: &str = "You help someone revise for a Microsoft certification exam. \
You are given one exam question, its options, its answer key and its explanation. \
Be concrete and short. Do not restate the question. Do not invent facts about the exam format. \
If the explanation contradicts the answer key, say so plainly instead of smoothing it over.";

pub fn build_prompt(task: Task, question: &QuestionDto) -> String {
    let options = question
        .options
        .iter()
        .map(|option| {
            format!(
                "{}. {}{}",
                option.letter,
                option.text,
                if option.is_correct {
                    "   [correct]"
                } else {
                    ""
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let matrix = if question.matrix.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nAnswer key recovered from the explanation:\n{}",
            question
                .matrix
                .iter()
                .map(|b| format!("Box {}: {}", b.index, b.value))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    let instruction = match task {
        Task::Explain => {
            "Explain why the correct option is correct, then name the single most tempting wrong \
             option and say what it would have been the answer to. Three short paragraphs at most."
        }
        Task::Variants => {
            "Write three new questions that test the same concept with different wording and \
             different distractors. Give each one four options and mark the correct letter. Do not \
             reuse the original phrasing."
        }
        Task::Note => {
            "Turn this into a study note of at most five bullet points: the rule to remember, not \
             a summary of this question."
        }
    };

    format!(
        "{instruction}\n\nQuestion #{number}\n{stem}\n\n{options}{matrix}\n\nExplanation from the \
         source:\n{explanation}",
        number = question.number,
        stem = question.stem,
        explanation = if question.explanation.is_empty() {
            "(none)"
        } else {
            &question.explanation
        }
    )
}

pub fn ask(source: Source, prompt: &str) -> AppResult<String> {
    match source {
        Source::Cli => ask_cli(prompt),
        Source::Anthropic => ask_anthropic(prompt),
    }
}

/// Two probes because the name resolves differently per machine: an `.exe` runs directly, while a
/// `.cmd` shim does not — `Command::new` will not execute a shim, so that case needs `cmd /C`.
fn locate_cli() -> Option<Vec<String>> {
    let probe = |program: &str, args: &[&str]| {
        crate::quiet(Command::new(program))
            .args(args)
            .arg("--version")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|_| {
                std::iter::once(program.to_string())
                    .chain(args.iter().map(|a| (*a).to_string()))
                    .collect::<Vec<_>>()
            })
    };

    #[cfg(windows)]
    {
        probe("claude", &[]).or_else(|| probe("cmd", &["/C", "claude"]))
    }
    #[cfg(not(windows))]
    {
        probe("claude", &[])
    }
}

fn ask_cli(prompt: &str) -> AppResult<String> {
    let command = locate_cli().ok_or_else(|| {
        AppError::Message(
            "the `claude` binary is not on PATH — install Claude Code, or switch the assistant to \
             a hosted provider in Settings"
                .into(),
        )
    })?;

    let (program, leading) = command.split_first().expect("probe returned a program");
    let output = crate::quiet(Command::new(program))
        .args(leading)
        .arg("-p")
        .arg(format!("{SYSTEM}\n\n{prompt}"))
        .output()
        .map_err(|error| AppError::Message(format!("could not run `claude`: {error}")))?;

    if !output.status.success() {
        return Err(AppError::Message(format!(
            "`claude` exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn ask_anthropic(prompt: &str) -> AppResult<String> {
    let key = entry()
        .and_then(|entry| entry.get_password())
        .map_err(|_| AppError::Message("no API key stored — add one in Settings".into()))?;

    let body = serde_json::json!({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        // Thinking is on by default on this model and counts against max_tokens. Low effort keeps
        // a one-question explanation from spending the whole budget on deliberation.
        "output_config": { "effort": "low" },
        "fallbacks": "default",
        "system": SYSTEM,
        "messages": [{ "role": "user", "content": prompt }],
    });

    let response = reqwest::blocking::Client::new()
        .post(API_URL)
        .header("x-api-key", key)
        .header("anthropic-version", API_VERSION)
        .header("anthropic-beta", BETA_FALLBACK)
        .json(&body)
        .send()
        .map_err(|error| AppError::Message(error.to_string()))?;

    let status = response.status();
    let payload: serde_json::Value = response
        .json()
        .map_err(|error| AppError::Message(error.to_string()))?;

    if !status.is_success() {
        let message = payload
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        return Err(AppError::Message(format!("{status}: {message}")));
    }

    // A safety classifier can decline the request: HTTP 200, no content, `stop_reason: refusal`.
    // Reading content[0] without this check yields an empty answer with no explanation.
    if payload.get("stop_reason").and_then(|v| v.as_str()) == Some("refusal") {
        return Err(AppError::Message(
            "the model declined this request; nothing was returned".into(),
        ));
    }

    let text = payload
        .get("content")
        .and_then(|content| content.as_array())
        .map(|blocks| {
            blocks
                .iter()
                .filter(|block| block.get("type").and_then(|t| t.as_str()) == Some("text"))
                .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    if text.trim().is_empty() {
        return Err(AppError::Message("the reply carried no text".into()));
    }
    Ok(text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::AnswerOption;
    use openexamtrainer_ingest::model::{MatrixBox, QuestionKind};

    fn question() -> QuestionDto {
        QuestionDto {
            id: 1,
            number: 7,
            topic: Some(1),
            kind: QuestionKind::Matrix,
            stem: "For each statement, select Yes or No.".into(),
            options: vec![
                AnswerOption {
                    letter: 'A',
                    text: "Mastered".into(),
                    is_correct: true,
                },
                AnswerOption {
                    letter: 'B',
                    text: "Not Mastered".into(),
                    is_correct: false,
                },
            ],
            answer_letters: vec!['A'],
            matrix: vec![MatrixBox {
                index: 1,
                value: "No".into(),
            }],
            explanation: "Anomaly detection covers fraud.".into(),
            references: Vec::new(),
            source_page: 2,
            confidence: 0.65,
            needs_source: true,
            warnings: Vec::new(),
            figures: Vec::new(),
        }
    }

    #[test]
    fn the_prompt_carries_the_answer_key_and_the_recovered_matrix() {
        let prompt = build_prompt(Task::Explain, &question());

        assert!(prompt.contains("A. Mastered   [correct]"));
        assert!(prompt.contains("Box 1: No"));
        assert!(prompt.contains("Anomaly detection covers fraud."));
        assert!(prompt.contains("most tempting wrong"));
    }

    #[test]
    fn each_task_asks_for_something_different() {
        let question = question();
        let explain = build_prompt(Task::Explain, &question);
        let variants = build_prompt(Task::Variants, &question);
        let note = build_prompt(Task::Note, &question);

        assert!(variants.contains("three new questions"));
        assert!(note.contains("five bullet points"));
        assert_ne!(explain, variants);
        assert_ne!(variants, note);
    }

    /// The transport rather than the prompt. Ignored by default: it needs the binary installed and
    /// it spends a real request, neither of which belongs in a check that runs on every commit.
    ///
    /// `cargo test --manifest-path src-tauri/Cargo.toml -- --ignored the_local_binary_answers`
    #[test]
    #[ignore = "runs the real `claude` binary and spends a request"]
    fn the_local_binary_answers() {
        let located = locate_cli();
        assert!(located.is_some(), "`claude` is not on PATH");

        let reply = ask(
            Source::Cli,
            "Answer with one short sentence: what is 2 + 2?",
        )
        .expect("the binary should answer");

        // Non-empty and exited zero is the whole claim: that the process is found, run, and read
        // back. What the model says is not this test's business.
        assert!(!reply.trim().is_empty(), "the reply carried no text");
        println!("probe: {located:?}\nreply: {reply}");
    }

    #[test]
    fn a_question_without_an_explanation_says_so_rather_than_leaving_a_gap() {
        let mut question = question();
        question.explanation = String::new();
        question.matrix.clear();

        let prompt = build_prompt(Task::Note, &question);

        assert!(prompt.contains("(none)"));
        assert!(!prompt.contains("Answer key recovered"));
    }
}

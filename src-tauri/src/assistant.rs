use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::eventlog::EventRecord;

const KEYRING_SERVICE: &str = "com.thorstenalpers.openeventviewer";
const KEYRING_ACCOUNT: &str = "anthropic-api-key";

const MODEL: &str = "claude-opus-5";
const MAX_TOKENS: u32 = 8192;
const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
/// Server-side fallbacks: when a safety classifier declines the request, the API re-runs it on a
/// substitute model instead of handing back an empty response.
const BETA_FALLBACK: &str = "server-side-fallback-2026-07-01";

/// What one event may contribute before it is cut short. A stack trace pasted into an event's data
/// can run to tens of kilobytes and would otherwise crowd out the forty events around it.
const PER_EVENT: usize = 2_000;

/// The whole rendered window. Well inside the model's context, and the point past which more events
/// stop adding evidence and start adding noise.
const TOTAL: usize = 120_000;

pub const SYSTEM: &str = "You are reading Windows event log records with someone who is trying to \
work out why their machine misbehaved. Lead with the most likely root cause and say how confident \
you are in it. Then say what to check or try next, concretely. Refer only to events you were \
given: never invent an event, an ID or a provider, and never assume an event exists because the \
story would be tidier with it. If the records do not support a conclusion, say that plainly and \
say which event would settle it. Be brief — this is a diagnosis, not an essay.";

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
pub enum Role {
    User,
    Assistant,
}

impl Role {
    fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub source: Source,
    pub cli_available: bool,
    pub has_key: bool,
    /// So the preview can show the whole of what is sent rather than only the part the user typed.
    pub system_prompt: String,
}

pub fn status(source: Source) -> Status {
    Status {
        source,
        cli_available: locate_cli().is_some(),
        has_key: entry().and_then(|e| e.get_password()).is_ok(),
        system_prompt: SYSTEM.to_string(),
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

/// Sends the conversation as it stands and returns the reply.
///
/// The messages are forwarded exactly as they arrive. Nothing is appended, summarised or rewritten
/// here, because the interface has already shown the user this string and promised it is what
/// leaves the machine.
pub fn chat(source: Source, messages: &[Message]) -> AppResult<String> {
    if messages.is_empty() {
        return Err(AppError::Message("there is nothing to send".into()));
    }
    match source {
        Source::Cli => chat_cli(messages),
        Source::Anthropic => chat_anthropic(messages),
    }
}

/// A whole conversation as one prompt, because the CLI takes one.
///
/// Pure, so the test can assert on what would be sent without running anything.
pub fn flatten_transcript(messages: &[Message]) -> String {
    let body = messages
        .iter()
        .map(|message| {
            let who = match message.role {
                Role::User => "User",
                Role::Assistant => "Assistant",
            };
            format!("{who}:\n{}", message.content)
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{SYSTEM}\n\n{body}")
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

/// The prompt goes in over stdin rather than as an argument.
///
/// A window of forty events is tens of kilobytes, and a command line on Windows stops at about
/// thirty-two thousand characters — as an argument this would fail on exactly the queries worth
/// asking about.
fn chat_cli(messages: &[Message]) -> AppResult<String> {
    let command = locate_cli().ok_or_else(|| {
        AppError::Message(
            "the `claude` binary is not on PATH — install Claude Code, or switch the assistant to \
             a hosted provider in Settings"
                .into(),
        )
    })?;

    let (program, leading) = command.split_first().expect("probe returned a program");
    let mut child = crate::quiet(Command::new(program))
        .args(leading)
        .arg("-p")
        .arg("--output-format")
        .arg("text")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| AppError::Message(format!("could not run `claude`: {error}")))?;

    let transcript = flatten_transcript(messages);
    child
        .stdin
        .take()
        .ok_or_else(|| AppError::Message("`claude` refused its own stdin".into()))?
        .write_all(transcript.as_bytes())
        .map_err(|error| AppError::Message(format!("could not write to `claude`: {error}")))?;

    let output = child
        .wait_with_output()
        .map_err(|error| AppError::Message(format!("`claude` did not finish: {error}")))?;

    if !output.status.success() {
        return Err(AppError::Message(format!(
            "`claude` exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn chat_anthropic(messages: &[Message]) -> AppResult<String> {
    let key = entry()
        .and_then(|entry| entry.get_password())
        .map_err(|_| AppError::Message("no API key stored — add one in Settings".into()))?;

    let wire: Vec<serde_json::Value> = messages
        .iter()
        .map(|message| {
            serde_json::json!({ "role": message.role.as_str(), "content": message.content })
        })
        .collect();

    let body = serde_json::json!({
        "model": MODEL,
        "max_tokens": MAX_TOKENS,
        // Thinking is on by default on this model and counts against max_tokens. Low effort keeps
        // a single diagnosis from spending the whole budget on deliberation.
        "output_config": { "effort": "low" },
        "fallbacks": "default",
        "system": SYSTEM,
        "messages": wire,
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

/// A run of events as the text the assistant is given, and the text the preview shows.
///
/// Oldest first, whatever order they arrived in: a diagnosis is a story about what happened next,
/// and a table sorted newest-first tells it backwards.
pub fn render_events_for_prompt(events: &[EventRecord]) -> String {
    let mut ordered: Vec<&EventRecord> = events.iter().collect();
    ordered.sort_by(|left, right| left.time_created.cmp(&right.time_created));

    let mut out = String::new();
    for (index, event) in ordered.iter().enumerate() {
        let block = one_event(event);
        if out.len() + block.len() > TOTAL {
            out.push_str(&format!(
                "\n… {} further events truncated to keep the prompt within its budget.\n",
                ordered.len() - index
            ));
            break;
        }
        out.push_str(&block);
    }
    out
}

fn one_event(event: &EventRecord) -> String {
    let mut block = format!(
        "[{}] {} | {} | {} | EventID {} | {}\n{}\n",
        event.time_created,
        event.channel,
        event.level_name,
        event.provider,
        event.event_id,
        event.computer,
        event.message.trim()
    );

    if !event.event_data.is_empty() {
        let data = event
            .event_data
            .iter()
            .map(|item| format!("{}={}", item.name, item.value))
            .collect::<Vec<_>>()
            .join(", ");
        block.push_str(&format!("data: {data}\n"));
    }

    if block.len() > PER_EVENT {
        // On a char boundary, or the truncation itself panics on the first non-ASCII message.
        let mut cut = PER_EVENT;
        while !block.is_char_boundary(cut) {
            cut -= 1;
        }
        block.truncate(cut);
        block.push_str("… (event truncated)\n");
    }

    block.push('\n');
    block
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eventlog::DataItem;

    fn event(time: &str, message: &str) -> EventRecord {
        EventRecord {
            record_id: 1,
            channel: "System".into(),
            provider: "Microsoft-Windows-Kernel-Power".into(),
            event_id: 41,
            level: 1,
            level_name: "Critical".into(),
            task: "None".into(),
            keywords: Vec::new(),
            time_created: time.into(),
            computer: "WORKBENCH".into(),
            message: message.into(),
            event_data: vec![DataItem {
                name: "BugcheckCode".into(),
                value: "0".into(),
            }],
        }
    }

    /// A diagnosis is a story about what happened next; a table sorted newest-first tells it
    /// backwards, and the assistant would read the cause as the consequence.
    #[test]
    fn events_are_rendered_oldest_first_whatever_order_they_arrived_in() {
        let rendered = render_events_for_prompt(&[
            event("2026-08-20T10:00:00.000Z", "second"),
            event("2026-08-20T09:00:00.000Z", "first"),
        ]);

        let first = rendered.find("first").expect("first is there");
        let second = rendered.find("second").expect("second is there");
        assert!(first < second, "{rendered}");
    }

    #[test]
    fn one_event_carries_its_header_its_message_and_its_data() {
        let rendered =
            render_events_for_prompt(&[event("2026-08-20T09:00:00.000Z", "it rebooted")]);

        assert!(rendered.contains(
            "[2026-08-20T09:00:00.000Z] System | Critical | Microsoft-Windows-Kernel-Power | \
             EventID 41 | WORKBENCH"
        ));
        assert!(rendered.contains("it rebooted"));
        assert!(rendered.contains("data: BugcheckCode=0"));
    }

    #[test]
    fn a_single_enormous_event_is_cut_short_rather_than_crowding_out_the_rest() {
        let rendered = render_events_for_prompt(&[event(
            "2026-08-20T09:00:00.000Z",
            &"x".repeat(PER_EVENT * 3),
        )]);

        assert!(rendered.contains("(event truncated)"));
        assert!(rendered.len() < PER_EVENT + 200);
    }

    #[test]
    fn a_window_past_the_budget_says_how_many_events_it_dropped() {
        let many: Vec<EventRecord> = (0..200)
            .map(|index| {
                event(
                    &format!("2026-08-20T09:00:{index:02}.000Z"),
                    &"y".repeat(PER_EVENT - 200),
                )
            })
            .collect();

        let rendered = render_events_for_prompt(&many);

        assert!(rendered.contains("further events truncated"));
        assert!(rendered.len() <= TOTAL + PER_EVENT);
    }

    /// The CLI takes one prompt, so the whole conversation has to become one string — and who said
    /// what has to survive that, or the model reads its own last answer as the user's question.
    #[test]
    fn a_transcript_keeps_the_system_prompt_and_says_who_spoke() {
        let flattened = flatten_transcript(&[
            Message {
                role: Role::User,
                content: "why did it reboot?".into(),
            },
            Message {
                role: Role::Assistant,
                content: "unexpected power loss".into(),
            },
            Message {
                role: Role::User,
                content: "what should I check?".into(),
            },
        ]);

        assert!(flattened.starts_with(SYSTEM));
        assert!(flattened.contains("User:\nwhy did it reboot?"));
        assert!(flattened.contains("Assistant:\nunexpected power loss"));
        assert!(flattened.ends_with("User:\nwhat should I check?"));
    }

    #[test]
    fn an_empty_conversation_is_refused_rather_than_sent() {
        assert!(chat(Source::Cli, &[]).is_err());
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

        let reply = chat(
            Source::Cli,
            &[Message {
                role: Role::User,
                content: "Answer with one short sentence: what is 2 + 2?".into(),
            }],
        )
        .expect("the binary should answer");

        // Non-empty and exited zero is the whole claim: that the process is found, run, fed over
        // stdin and read back. What the model says is not this test's business.
        assert!(!reply.trim().is_empty(), "the reply carried no text");
        println!("probe: {located:?}\nreply: {reply}");
    }
}

//! The only file that touches Win32.
//!
//! Everything the rest of the app needs from the event log arrives as plain data through `query`
//! and `list_channels`. The parts that decide anything — how a filter becomes an XPath, how an
//! event's XML becomes a record — are pure functions below, so they can be tested without a
//! machine that happens to have the right entries in its log.

use std::collections::HashMap;
use std::ffi::c_void;

use serde::{Deserialize, Serialize};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_EVT_MAX_INSERTS_REACHED, ERROR_EVT_UNRESOLVED_PARAMETER_INSERT,
    ERROR_EVT_UNRESOLVED_VALUE_INSERT, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS,
    ERROR_TIMEOUT, WIN32_ERROR,
};
use windows::Win32::System::EventLog::{
    EvtClose, EvtFormatMessage, EvtFormatMessageEvent, EvtFormatMessageTask, EvtNext,
    EvtNextChannelPath, EvtOpenChannelEnum, EvtOpenPublisherMetadata, EvtQuery,
    EvtQueryChannelPath, EvtQueryReverseDirection, EvtQueryTolerateQueryErrors, EvtRender,
    EvtRenderEventXml, EVT_HANDLE,
};

use crate::error::{AppError, AppResult};

/// The most a single query may return, whatever the caller asks for. Every event costs a message
/// lookup, and past this the wait stops being a wait and becomes a hang.
const MAX_EVENTS: usize = 50_000;

/// How many handles one `EvtNext` call collects. Larger batches save round trips into wevtapi and
/// cost nothing but stack.
const BATCH: usize = 128;

/// How long `EvtNext` may wait for the next batch.
const NEXT_TIMEOUT_MS: u32 = 2_000;

/// What wevtapi will accept in one query. Documented, and enforced here rather than by the API:
/// with `EvtQueryTolerateQueryErrors` an over-long query is not rejected, it simply matches
/// nothing — the one failure mode that looks exactly like an empty log.
const MAX_EXPRESSIONS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataItem {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRecord {
    pub record_id: u64,
    pub channel: String,
    pub provider: String,
    pub event_id: u32,
    pub level: u8,
    pub level_name: String,
    pub task: String,
    pub keywords: Vec<String>,
    /// RFC 3339, UTC, milliseconds. Windows records a hundred-nanosecond tick; nothing above this
    /// layer sorts finely enough for the rest of it to mean anything.
    pub time_created: String,
    pub computer: String,
    pub message: String,
    pub event_data: Vec<DataItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    #[serde(default)]
    pub channels: Vec<String>,
    #[serde(default)]
    pub levels: Vec<u8>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub event_ids: Vec<u32>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub max: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub events: Vec<EventRecord>,
    /// The log held more than the cap allowed. Said plainly rather than implied by a round count.
    pub truncated: bool,
    pub elapsed_ms: u64,
}

/// Everything an event's own XML carries. The message is not in here: it lives in the publisher's
/// resources and has to be asked for separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub record_id: u64,
    pub channel: String,
    pub provider: String,
    pub event_id: u32,
    pub level: u8,
    pub task_id: u32,
    pub keywords: u64,
    pub time_created: String,
    pub computer: String,
    pub event_data: Vec<DataItem>,
}

struct Handle(EVT_HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = EvtClose(self.0);
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn is(error: &windows::core::Error, code: WIN32_ERROR) -> bool {
    error.code() == code.to_hresult()
}

/// Turns a wevtapi failure into something a reader can act on.
///
/// The Security channel is the one that fails on a normal account, and "access denied" without the
/// reason reads as a bug in the app rather than as the operating system doing its job.
fn describe(error: &windows::core::Error, what: &str) -> AppError {
    if is(error, ERROR_ACCESS_DENIED) {
        return AppError::Message(format!(
            "{what} needs administrator rights — the Security channel is readable only by an \
             elevated process. Start OpenEventViewer as administrator to read it."
        ));
    }
    match error.code().0 as u32 & 0xffff {
        15007 => AppError::Message(format!("{what}: there is no such channel")),
        15001 => AppError::Message(format!("{what}: the query is not valid")),
        _ => AppError::Message(format!("{what}: {error}")),
    }
}

/// Every channel this machine publishes, so the toolbar can offer more than the four everybody
/// knows about.
pub fn list_channels() -> AppResult<Vec<String>> {
    let mut channels = Vec::new();
    let enumerator = Handle(
        unsafe { EvtOpenChannelEnum(None, 0) }
            .map_err(|error| describe(&error, "listing the channels"))?,
    );

    loop {
        let mut used = 0u32;
        let first = unsafe { EvtNextChannelPath(enumerator.0, None, &mut used) };
        if let Err(error) = &first {
            if is(error, ERROR_NO_MORE_ITEMS) {
                break;
            }
            if !is(error, ERROR_INSUFFICIENT_BUFFER) {
                return Err(describe(error, "listing the channels"));
            }
        }

        let mut buffer = vec![0u16; used as usize];
        unsafe { EvtNextChannelPath(enumerator.0, Some(&mut buffer), &mut used) }
            .map_err(|error| describe(&error, "listing the channels"))?;
        channels.push(from_wide(&buffer));
    }

    channels.sort_unstable();
    Ok(channels)
}

pub fn query(filter: &Filter) -> AppResult<QueryResult> {
    let started = std::time::Instant::now();
    let wanted = filter.max.clamp(1, MAX_EVENTS);

    let conditions = expression_count(filter);
    if conditions > MAX_EXPRESSIONS {
        return Err(AppError::Message(format!(
            "this filter asks {conditions} separate conditions and the event log accepts at most \
             {MAX_EXPRESSIONS} — narrow the levels, the event ids or the providers"
        )));
    }

    let text = build_query(filter);

    // One channel goes in as a path with a plain XPath; anything else has to be a structured query,
    // and a structured query insists the path be null.
    let single = (filter.channels.len() == 1).then(|| wide(&filter.channels[0]));
    let path = single
        .as_ref()
        .map_or(PCWSTR::null(), |value| PCWSTR(value.as_ptr()));
    let query_text = wide(&text);

    let what = if filter.channels.is_empty() {
        "reading the event log".to_string()
    } else {
        format!("reading {}", filter.channels.join(", "))
    };

    let results = Handle(
        unsafe {
            EvtQuery(
                None,
                path,
                PCWSTR(query_text.as_ptr()),
                EvtQueryChannelPath.0 | EvtQueryReverseDirection.0 | EvtQueryTolerateQueryErrors.0,
            )
        }
        .map_err(|error| describe(&error, &what))?,
    );

    // One publisher's metadata handle serves every event it wrote. Opening it per event is what
    // turns a five-second query into a minute of them.
    let mut publishers: HashMap<String, Option<Handle>> = HashMap::new();
    let mut events = Vec::new();
    // One past the cap, so a full page can say whether there was more behind it.
    let ceiling = wanted + 1;

    'outer: while events.len() < ceiling {
        let mut batch = [0isize; BATCH];
        let mut returned = 0u32;
        let room = (ceiling - events.len()).min(BATCH);

        let next = unsafe {
            EvtNext(
                results.0,
                &mut batch[..room],
                NEXT_TIMEOUT_MS,
                0,
                &mut returned,
            )
        };
        if let Err(error) = &next {
            if is(error, ERROR_NO_MORE_ITEMS) || is(error, ERROR_TIMEOUT) {
                break;
            }
            return Err(describe(error, &what));
        }

        for raw in batch.iter().take(returned as usize) {
            let event = Handle(EVT_HANDLE(*raw));
            let xml = match render_xml(event.0) {
                Ok(xml) => xml,
                // One unreadable event must not lose the other forty thousand.
                Err(_) => continue,
            };
            let Ok(parsed) = parse_event_xml(&xml) else {
                continue;
            };

            let publisher = publishers
                .entry(parsed.provider.clone())
                .or_insert_with(|| open_publisher(&parsed.provider));
            let handle = publisher.as_ref().map(|held| held.0);

            let message = format_message(handle, event.0, EvtFormatMessageEvent.0)
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| {
                    fallback_message(&parsed.provider, parsed.event_id, &parsed.event_data)
                });
            let task = format_message(handle, event.0, EvtFormatMessageTask.0)
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| task_name(parsed.task_id));

            events.push(EventRecord {
                record_id: parsed.record_id,
                channel: parsed.channel,
                provider: parsed.provider,
                event_id: parsed.event_id,
                level: parsed.level,
                level_name: level_name(parsed.level).to_string(),
                task,
                keywords: keyword_names(parsed.keywords),
                time_created: parsed.time_created,
                computer: parsed.computer,
                message,
                event_data: parsed.event_data,
            });

            if events.len() == ceiling {
                break 'outer;
            }
        }

        if (returned as usize) < room {
            break;
        }
    }

    let truncated = events.len() > wanted;
    events.truncate(wanted);
    // A reverse query per channel is merged by wevtapi channel by channel rather than by time, so
    // a multi-channel result arrives grouped. Newest first is what the table promises.
    events.sort_by(|left, right| right.time_created.cmp(&left.time_created));

    Ok(QueryResult {
        events,
        truncated,
        elapsed_ms: started.elapsed().as_millis() as u64,
    })
}

/// One event's raw XML, fetched on demand rather than kept for every row.
pub fn event_xml(channel: &str, record_id: u64) -> AppResult<String> {
    let path = wide(channel);
    let text = wide(&format!("*[System[EventRecordID={record_id}]]"));
    let what = format!("reading event {record_id} from {channel}");

    let results = Handle(
        unsafe {
            EvtQuery(
                None,
                PCWSTR(path.as_ptr()),
                PCWSTR(text.as_ptr()),
                EvtQueryChannelPath.0 | EvtQueryTolerateQueryErrors.0,
            )
        }
        .map_err(|error| describe(&error, &what))?,
    );

    let mut batch = [0isize; 1];
    let mut returned = 0u32;
    unsafe { EvtNext(results.0, &mut batch, NEXT_TIMEOUT_MS, 0, &mut returned) }
        .map_err(|error| describe(&error, &what))?;
    if returned == 0 {
        return Err(AppError::Message(format!(
            "{channel} no longer holds event {record_id}"
        )));
    }

    let event = Handle(EVT_HANDLE(batch[0]));
    render_xml(event.0).map_err(|error| describe(&error, &what))
}

fn open_publisher(provider: &str) -> Option<Handle> {
    let name = wide(provider);
    unsafe { EvtOpenPublisherMetadata(None, PCWSTR(name.as_ptr()), PCWSTR::null(), 0, 0) }
        .ok()
        .map(Handle)
}

/// The two-call buffer dance wevtapi wants: ask with nothing, be told how much, ask again.
fn render_xml(event: EVT_HANDLE) -> windows::core::Result<String> {
    let mut used = 0u32;
    let mut properties = 0u32;
    let first = unsafe {
        EvtRender(
            None,
            event,
            EvtRenderEventXml.0,
            0,
            None,
            &mut used,
            &mut properties,
        )
    };
    match first {
        Ok(()) => return Ok(String::new()),
        Err(error) if !is(&error, ERROR_INSUFFICIENT_BUFFER) => return Err(error),
        Err(_) => {}
    }

    // `used` is bytes here, unlike EvtFormatMessage, which counts characters.
    let mut buffer = vec![0u16; used as usize / 2 + 1];
    unsafe {
        EvtRender(
            None,
            event,
            EvtRenderEventXml.0,
            used,
            Some(buffer.as_mut_ptr() as *mut c_void),
            &mut used,
            &mut properties,
        )
    }?;
    Ok(from_wide(&buffer))
}

/// The publisher's own text for an event, where it has one.
///
/// An unresolved insert is a success: the publisher's template referenced a value the event does
/// not carry, and the rest of the sentence is still the best description that exists.
fn format_message(publisher: Option<EVT_HANDLE>, event: EVT_HANDLE, flag: u32) -> Option<String> {
    let tolerable = |error: &windows::core::Error| {
        is(error, ERROR_EVT_UNRESOLVED_VALUE_INSERT)
            || is(error, ERROR_EVT_UNRESOLVED_PARAMETER_INSERT)
            || is(error, ERROR_EVT_MAX_INSERTS_REACHED)
    };

    let mut used = 0u32;
    let first = unsafe { EvtFormatMessage(publisher, Some(event), 0, None, flag, None, &mut used) };
    match &first {
        Ok(()) => return Some(String::new()),
        Err(error) if is(error, ERROR_INSUFFICIENT_BUFFER) || tolerable(error) => {}
        Err(_) => return None,
    }
    if used == 0 {
        return None;
    }

    let mut buffer = vec![0u16; used as usize];
    let second = unsafe {
        EvtFormatMessage(
            publisher,
            Some(event),
            0,
            None,
            flag,
            Some(&mut buffer),
            &mut used,
        )
    };
    match second {
        Ok(()) => Some(from_wide(&buffer)),
        Err(error) if tolerable(&error) => Some(from_wide(&buffer)),
        Err(_) => None,
    }
}

fn from_wide(buffer: &[u16]) -> String {
    let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
        .trim_end()
        .to_string()
}

/// What to show when the publisher has no message for the event.
///
/// The numbers and the raw data rather than an apology: an event whose provider is uninstalled is
/// still evidence, and its `EventData` is what a search engine is given anyway.
pub fn fallback_message(provider: &str, event_id: u32, data: &[DataItem]) -> String {
    if data.is_empty() {
        return format!("{provider} event {event_id} — the publisher has no description for it.");
    }
    let joined = data
        .iter()
        .map(|item| format!("{}={}", item.name, item.value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{provider} event {event_id} — no description; recorded data: {joined}")
}

pub fn level_name(level: u8) -> &'static str {
    match level {
        1 => "Critical",
        2 => "Error",
        3 => "Warning",
        5 => "Verbose",
        // 0 is "log always", which every viewer since Vista has shown as Information.
        _ => "Information",
    }
}

fn task_name(task_id: u32) -> String {
    if task_id == 0 {
        "None".to_string()
    } else {
        format!("Task {task_id}")
    }
}

/// The reserved bits of the keyword mask, which are the only ones whose meaning is the same for
/// every publisher. A publisher's own keywords live in its metadata and are not worth a lookup per
/// event for a column nobody sorts by.
pub fn keyword_names(mask: u64) -> Vec<String> {
    const RESERVED: [(u64, &str); 7] = [
        (0x0001_0000_0000_0000, "Response Time"),
        (0x0002_0000_0000_0000, "WDI Diagnostic"),
        (0x0004_0000_0000_0000, "SQM"),
        (0x0010_0000_0000_0000, "Audit Failure"),
        (0x0020_0000_0000_0000, "Audit Success"),
        (0x0040_0000_0000_0000, "Correlation Hint"),
        (0x0080_0000_0000_0000, "Classic"),
    ];

    RESERVED
        .iter()
        .filter(|(bit, _)| mask & bit != 0)
        .map(|(_, name)| (*name).to_string())
        .collect()
}

/// A filter as wevtapi wants to hear it.
///
/// One channel is a plain XPath against the path the query is opened on. Anything else — none, or
/// several — has to be a structured query, because a path can only name one channel.
pub fn build_query(filter: &Filter) -> String {
    match filter.channels.len() {
        1 => predicate(filter, false),
        _ => {
            let channels: Vec<&str> = if filter.channels.is_empty() {
                vec!["System", "Application"]
            } else {
                filter.channels.iter().map(String::as_str).collect()
            };
            let inner = predicate(filter, true);
            let selects = channels
                .iter()
                .map(|channel| {
                    format!(
                        "<Select Path=\"{}\">{inner}</Select>",
                        escape(channel, true)
                    )
                })
                .collect::<String>();
            format!("<QueryList><Query Id=\"0\">{selects}</Query></QueryList>")
        }
    }
}

/// The comparison operators are the whole reason this needs to know where it will end up.
///
/// A structured query is XML, so `>` has to be written `&gt;`. A single-channel query is not: the
/// string goes straight to wevtapi, which reads `&gt;` as three characters and quietly matches
/// nothing at all — a time filter that silently returns an empty log.
fn predicate(filter: &Filter, xml: bool) -> String {
    let (gt, lt) = if xml { ("&gt;", "&lt;") } else { (">", "<") };
    let mut clauses: Vec<String> = Vec::new();

    if !filter.levels.is_empty() {
        let mut levels: Vec<u8> = filter.levels.clone();
        // Information is written as 4, but the kernel writes "log always" as 0 and every viewer
        // shows it in the same row — asking for one without the other loses events silently.
        if levels.contains(&4) && !levels.contains(&0) {
            levels.push(0);
        }
        levels.sort_unstable();
        levels.dedup();
        clauses.push(one_of(levels.iter().map(|level| format!("Level={level}"))));
    }

    if let Some(from) = filter.from.as_deref().filter(|value| !value.is_empty()) {
        clauses.push(format!(
            "TimeCreated[@SystemTime{gt}='{}']",
            escape(from, xml)
        ));
    }
    if let Some(to) = filter.to.as_deref().filter(|value| !value.is_empty()) {
        clauses.push(format!(
            "TimeCreated[@SystemTime{lt}='{}']",
            escape(to, xml)
        ));
    }

    if !filter.event_ids.is_empty() {
        clauses.push(one_of(
            filter.event_ids.iter().map(|id| format!("EventID={id}")),
        ));
    }

    if !filter.providers.is_empty() {
        let names = filter
            .providers
            .iter()
            .map(|name| format!("@Name=\"{}\"", escape(name, xml)))
            .collect::<Vec<_>>()
            .join(" or ");
        clauses.push(format!("Provider[{names}]"));
    }

    if clauses.is_empty() {
        "*".to_string()
    } else {
        format!("*[System[{}]]", clauses.join(" and "))
    }
}

/// Every term wevtapi counts against its own ceiling.
pub fn expression_count(filter: &Filter) -> usize {
    let levels = if filter.levels.is_empty() {
        0
    } else if filter.levels.contains(&4) && !filter.levels.contains(&0) {
        filter.levels.len() + 1
    } else {
        filter.levels.len()
    };

    levels
        + usize::from(
            filter
                .from
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
        )
        + usize::from(filter.to.as_deref().is_some_and(|value| !value.is_empty()))
        + filter.event_ids.len()
        + filter.providers.len()
}

fn one_of(parts: impl Iterator<Item = String>) -> String {
    let joined: Vec<String> = parts.collect();
    if joined.len() == 1 {
        joined.into_iter().next().unwrap_or_default()
    } else {
        format!("({})", joined.join(" or "))
    }
}

/// A provider name arrives from a text field, so it is the one part of the query nothing has
/// vetted.
///
/// The literal is double-quoted, which is why the double quote is the character that has to go: an
/// apostrophe is then harmless in both forms, and an XPath string literal has no escape of its own
/// to fall back on. The XML entities apply only where the query is embedded in XML.
fn escape(value: &str, xml: bool) -> String {
    let stripped = value.replace('"', "");
    if !xml {
        return stripped;
    }
    stripped
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// One event's XML as the record the rest of the app works with.
pub fn parse_event_xml(xml: &str) -> AppResult<Parsed> {
    let document = roxmltree::Document::parse(xml)
        .map_err(|error| AppError::Message(format!("event xml: {error}")))?;
    let root = document.root_element();

    let system = root
        .children()
        .find(|node| node.has_tag_name("System"))
        .ok_or_else(|| AppError::Message("event xml carries no System block".into()))?;

    let child = |name: &str| system.children().find(|node| node.has_tag_name(name));
    let text = |name: &str| {
        child(name)
            .and_then(|node| node.text())
            .unwrap_or_default()
            .trim()
            .to_string()
    };

    let keywords = child("Keywords")
        .and_then(|node| node.text())
        .map(|value| value.trim().trim_start_matches("0x"))
        .and_then(|value| u64::from_str_radix(value, 16).ok())
        .unwrap_or(0);

    Ok(Parsed {
        record_id: text("EventRecordID").parse().unwrap_or(0),
        channel: text("Channel"),
        provider: child("Provider")
            .and_then(|node| node.attribute("Name"))
            .unwrap_or_default()
            .to_string(),
        event_id: text("EventID").parse().unwrap_or(0),
        level: text("Level").parse().unwrap_or(0),
        task_id: text("Task").parse().unwrap_or(0),
        keywords,
        time_created: to_millis(
            child("TimeCreated")
                .and_then(|node| node.attribute("SystemTime"))
                .unwrap_or_default(),
        ),
        computer: text("Computer"),
        event_data: read_data(&root),
    })
}

/// The payload, whichever of the two shapes a publisher chose.
///
/// `EventData` is a flat list whose entries may or may not be named; `UserData` is a tree of the
/// publisher's own design. Both end up as name/value pairs, because that is what a detail pane can
/// show and a prompt can carry.
fn read_data(root: &roxmltree::Node<'_, '_>) -> Vec<DataItem> {
    let mut items = Vec::new();

    if let Some(block) = root.children().find(|node| node.has_tag_name("EventData")) {
        for (index, node) in block
            .children()
            .filter(|node| node.has_tag_name("Data"))
            .enumerate()
        {
            items.push(DataItem {
                name: node
                    .attribute("Name")
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("Data{}", index + 1)),
                value: node.text().unwrap_or_default().trim().to_string(),
            });
        }
    }

    if let Some(block) = root.children().find(|node| node.has_tag_name("UserData")) {
        for node in block.descendants().filter(|node| node.is_element()) {
            let Some(value) = node.text().map(str::trim).filter(|text| !text.is_empty()) else {
                continue;
            };
            items.push(DataItem {
                name: node.tag_name().name().to_string(),
                value: value.to_string(),
            });
        }
    }

    items
}

/// Windows stamps a hundred-nanosecond tick; this keeps three digits of it.
///
/// Truncation rather than rounding, so a timestamp never moves into the second after the one the
/// event was written in.
pub fn to_millis(stamp: &str) -> String {
    let stamp = stamp.trim();
    let Some((seconds, rest)) = stamp.split_once('.') else {
        return stamp.to_string();
    };
    let fraction: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let zone = &rest[fraction.len()..];
    let millis: String = fraction
        .chars()
        .chain(std::iter::repeat('0'))
        .take(3)
        .collect();
    format!(
        "{seconds}.{millis}{}",
        if zone.is_empty() { "Z" } else { zone }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter() -> Filter {
        Filter {
            max: 100,
            ..Filter::default()
        }
    }

    #[test]
    fn a_filter_that_asks_for_nothing_selects_everything() {
        let mut one = filter();
        one.channels = vec!["System".into()];

        assert_eq!(build_query(&one), "*");
    }

    #[test]
    fn one_channel_is_a_plain_xpath_and_several_are_a_query_list() {
        let mut one = filter();
        one.channels = vec!["System".into()];
        one.event_ids = vec![41];
        assert_eq!(build_query(&one), "*[System[EventID=41]]");

        let mut two = filter();
        two.channels = vec!["System".into(), "Application".into()];
        two.event_ids = vec![41];
        assert_eq!(
            build_query(&two),
            "<QueryList><Query Id=\"0\">\
             <Select Path=\"System\">*[System[EventID=41]]</Select>\
             <Select Path=\"Application\">*[System[EventID=41]]</Select>\
             </Query></QueryList>"
        );
    }

    #[test]
    fn no_channel_at_all_still_reads_the_two_that_matter() {
        let query = build_query(&filter());

        assert!(query.contains("Path=\"System\""));
        assert!(query.contains("Path=\"Application\""));
    }

    /// The kernel writes "log always" as level 0 and every viewer files it under Information.
    /// Asking for 4 alone would drop those events without saying so.
    #[test]
    fn asking_for_information_also_asks_for_level_zero() {
        let mut one = filter();
        one.channels = vec!["System".into()];
        one.levels = vec![4];

        assert_eq!(build_query(&one), "*[System[(Level=0 or Level=4)]]");
    }

    /// A single-channel query is a bare XPath, not XML. wevtapi reads an escaped `&gt;` as three
    /// characters and matches nothing at all — a time filter that silently empties the log.
    #[test]
    fn a_time_window_on_one_channel_compares_with_a_raw_operator() {
        let mut one = filter();
        one.channels = vec!["System".into()];
        one.from = Some("2026-08-20T00:00:00.000Z".into());
        one.to = Some("2026-08-21T00:00:00.000Z".into());

        assert_eq!(
            build_query(&one),
            "*[System[TimeCreated[@SystemTime>='2026-08-20T00:00:00.000Z'] and \
             TimeCreated[@SystemTime<='2026-08-21T00:00:00.000Z']]]"
        );
    }

    /// Embedded in a `<Select>`, the same comparison is XML and has to be escaped.
    #[test]
    fn a_time_window_across_channels_escapes_it_because_that_query_is_xml() {
        let mut two = filter();
        two.channels = vec!["System".into(), "Application".into()];
        two.from = Some("2026-08-20T00:00:00.000Z".into());

        let query = build_query(&two);

        assert!(
            query.contains("@SystemTime&gt;='2026-08-20T00:00:00.000Z'"),
            "{query}"
        );
        assert!(!query.contains("@SystemTime>="), "{query}");
    }

    #[test]
    fn several_event_ids_and_providers_are_alternatives_within_one_clause() {
        let mut one = filter();
        one.channels = vec!["System".into()];
        one.event_ids = vec![41, 6008];
        one.providers = vec!["Microsoft-Windows-Kernel-Power".into()];

        assert_eq!(
            build_query(&one),
            "*[System[(EventID=41 or EventID=6008) and \
             Provider[@Name=\"Microsoft-Windows-Kernel-Power\"]]]"
        );
    }

    /// The literal is double-quoted, so an apostrophe in a provider name is harmless and a double
    /// quote is the one character that would close it.
    #[test]
    fn a_quote_in_a_provider_name_cannot_close_the_literal() {
        let mut one = filter();
        one.channels = vec!["System".into()];
        one.providers = vec![r#"O'Brien "Software""#.into()];

        assert_eq!(
            build_query(&one),
            r#"*[System[Provider[@Name="O'Brien Software"]]]"#
        );
    }

    const NAMED: &str = r#"<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <Provider Name="Microsoft-Windows-Kernel-Power" Guid="{331c3b3a}" />
    <EventID>41</EventID>
    <Level>1</Level>
    <Task>63</Task>
    <Keywords>0x8000400000000002</Keywords>
    <TimeCreated SystemTime="2026-08-20T09:13:22.1234567Z" />
    <EventRecordID>90210</EventRecordID>
    <Channel>System</Channel>
    <Computer>WORKBENCH</Computer>
  </System>
  <EventData>
    <Data Name="BugcheckCode">0</Data>
    <Data Name="SleepInProgress">0</Data>
  </EventData>
</Event>"#;

    const UNNAMED: &str = r#"<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <Provider Name="EventLog" />
    <EventID>6008</EventID>
    <Level>2</Level>
    <Task>0</Task>
    <Keywords>0x0080000000000000</Keywords>
    <TimeCreated SystemTime="2026-08-19T21:04:00.5Z" />
    <EventRecordID>7</EventRecordID>
    <Channel>System</Channel>
    <Computer>WORKBENCH</Computer>
  </System>
  <EventData>
    <Data>21:03:11</Data>
    <Data>19.08.2026</Data>
  </EventData>
</Event>"#;

    const USER_DATA: &str = r#"<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <Provider Name="Microsoft-Windows-Winlogon" />
    <EventID>6005</EventID>
    <Level>4</Level>
    <Task>0</Task>
    <Keywords>0x0000000000000000</Keywords>
    <TimeCreated SystemTime="2026-08-18T06:00:00Z" />
    <EventRecordID>3</EventRecordID>
    <Channel>Application</Channel>
    <Computer>WORKBENCH</Computer>
  </System>
  <UserData>
    <EventXML><Reason>Startup</Reason><Seconds>12</Seconds></EventXML>
  </UserData>
</Event>"#;

    #[test]
    fn a_named_payload_keeps_the_publishers_own_names() {
        let parsed = parse_event_xml(NAMED).expect("parses");

        assert_eq!(parsed.record_id, 90210);
        assert_eq!(parsed.provider, "Microsoft-Windows-Kernel-Power");
        assert_eq!(parsed.event_id, 41);
        assert_eq!(parsed.level, 1);
        assert_eq!(parsed.channel, "System");
        assert_eq!(parsed.computer, "WORKBENCH");
        assert_eq!(parsed.keywords, 0x8000_4000_0000_0002);
        assert_eq!(parsed.time_created, "2026-08-20T09:13:22.123Z");
        assert_eq!(
            parsed.event_data,
            vec![
                DataItem {
                    name: "BugcheckCode".into(),
                    value: "0".into()
                },
                DataItem {
                    name: "SleepInProgress".into(),
                    value: "0".into()
                }
            ]
        );
    }

    /// An unnamed `<Data>` is common in the classic providers. Numbering them keeps the order,
    /// which is the only thing that tells them apart.
    #[test]
    fn an_unnamed_payload_is_numbered_in_order() {
        let parsed = parse_event_xml(UNNAMED).expect("parses");

        assert_eq!(parsed.provider, "EventLog");
        assert_eq!(parsed.time_created, "2026-08-19T21:04:00.500Z");
        assert_eq!(
            parsed.event_data,
            vec![
                DataItem {
                    name: "Data1".into(),
                    value: "21:03:11".into()
                },
                DataItem {
                    name: "Data2".into(),
                    value: "19.08.2026".into()
                }
            ]
        );
    }

    #[test]
    fn a_user_data_tree_is_flattened_to_the_leaves_that_carry_text() {
        let parsed = parse_event_xml(USER_DATA).expect("parses");

        assert_eq!(parsed.time_created, "2026-08-18T06:00:00Z");
        assert_eq!(
            parsed.event_data,
            vec![
                DataItem {
                    name: "Reason".into(),
                    value: "Startup".into()
                },
                DataItem {
                    name: "Seconds".into(),
                    value: "12".into()
                }
            ]
        );
    }

    #[test]
    fn a_fallback_message_names_the_event_and_prints_what_was_recorded() {
        let data = vec![DataItem {
            name: "BugcheckCode".into(),
            value: "26".into(),
        }];

        assert_eq!(
            fallback_message("Contoso-Agent", 4711, &data),
            "Contoso-Agent event 4711 — no description; recorded data: BugcheckCode=26"
        );
        assert_eq!(
            fallback_message("Contoso-Agent", 4711, &[]),
            "Contoso-Agent event 4711 — the publisher has no description for it."
        );
    }

    #[test]
    fn the_reserved_keyword_bits_are_named_and_the_rest_are_left_alone() {
        assert_eq!(keyword_names(0x0080_0000_0000_0000), vec!["Classic"]);
        assert_eq!(keyword_names(0x0020_0000_0000_0000), vec!["Audit Success"]);
        assert!(keyword_names(0x0000_0000_0000_0002).is_empty());
    }

    /// An over-long query is the worst kind of failure: `EvtQueryTolerateQueryErrors` means
    /// wevtapi does not reject it, it just matches nothing, which reads as an empty log.
    #[test]
    fn a_filter_with_too_many_conditions_is_refused_rather_than_silently_emptied() {
        let mut one = filter();
        one.channels = vec!["System".into()];
        one.levels = vec![1, 2, 3];
        one.from = Some("2026-08-20T00:00:00.000Z".into());
        one.event_ids = (1..=20).collect();

        assert_eq!(expression_count(&one), 24);
        let refused = query(&one).expect_err("24 conditions is past the ceiling");
        assert!(refused.to_string().contains("at most 20"), "{refused}");
    }

    #[test]
    fn the_level_zero_that_information_drags_in_counts_towards_the_ceiling() {
        let mut one = filter();
        one.channels = vec!["System".into()];
        one.levels = vec![4];

        assert_eq!(expression_count(&one), 2);
    }

    #[test]
    fn levels_are_named_the_way_a_reader_expects_them() {
        assert_eq!(level_name(0), "Information");
        assert_eq!(level_name(1), "Critical");
        assert_eq!(level_name(2), "Error");
        assert_eq!(level_name(3), "Warning");
        assert_eq!(level_name(4), "Information");
        assert_eq!(level_name(5), "Verbose");
    }

    /// Ignored: it reads this machine's own System log, which a fresh runner has no useful entries
    /// in and which says nothing about the code when it is empty.
    ///
    /// `cargo test --manifest-path src-tauri/Cargo.toml -- --ignored the_system_log_answers`
    #[test]
    #[ignore = "reads the machine's own System log"]
    fn the_system_log_answers() {
        let result = query(&Filter {
            channels: vec!["System".into()],
            max: 10,
            ..Filter::default()
        })
        .expect("the System log should answer");

        assert!(!result.events.is_empty(), "no events came back");
        for event in &result.events {
            assert!(!event.time_created.is_empty());
            assert!(!event.provider.is_empty());
        }
        println!(
            "{} events in {} ms; newest: {:?}",
            result.events.len(),
            result.elapsed_ms,
            result.events.first()
        );
    }

    #[test]
    #[ignore = "reads the machine's own channel list"]
    fn the_machine_lists_its_channels() {
        let channels = list_channels().expect("channels");

        assert!(channels.iter().any(|name| name == "System"));
        assert!(channels.iter().any(|name| name == "Application"));
        println!("{} channels", channels.len());
    }
}

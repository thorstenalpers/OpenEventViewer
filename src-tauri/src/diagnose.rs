//! Finding the events worth explaining, and the ones around them.
//!
//! Everything that decides anything here is pure: which events count as an incident, how they
//! collapse into one, and what window of the log surrounds them. Only `incidents` and `bundle`
//! touch the log, and they only ask `eventlog` for it.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::assistant;
use crate::error::{AppError, AppResult};
use crate::eventlog::{self, EventRecord, Filter};

/// How far back the bundle reaches. Long enough to hold what led up to a freeze, short enough that
/// the run is still about one thing.
const BEFORE_MINUTES: i64 = 15;

/// And how far past it. The machine is usually already down; what little follows is the reboot.
const AFTER_MINUTES: i64 = 2;

/// Two of the same kind this close together are one thing happening, not two.
const COLLAPSE_SECONDS: i64 = 60;

const SCAN_MAX: usize = 2_000;
const BUNDLE_MAX: usize = 500;

/// The channels a bundle reads. The last two are where a freeze leaves its fingerprints when the
/// System log has nothing to say about it.
const BUNDLE_CHANNELS: [&str; 4] = [
    "System",
    "Application",
    "Microsoft-Windows-Diagnostics-Performance/Operational",
    "Microsoft-Windows-Kernel-Power/Thermal-Operational",
];

/// Every machine records thousands of these and none of them explains a crash. Left out of the
/// bundle so the run the assistant reads is evidence rather than wallpaper.
const NOISE: [&str; 2] = ["DCOM", "Microsoft-Windows-DistributedCOM"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    UnexpectedShutdown,
    BugCheck,
    HardwareError,
    AppHang,
    AppCrash,
    ServiceFailure,
    DiskError,
    Ntfs,
    DisplayTdr,
    ProcessorPower,
}

struct Signature {
    provider: &'static str,
    ids: &'static [u32],
    kind: Kind,
}

/// What a machine writes down when something went wrong, by the provider and id it writes it under.
const SIGNATURES: &[Signature] = &[
    Signature {
        provider: "Microsoft-Windows-Kernel-Power",
        ids: &[41, 137],
        kind: Kind::UnexpectedShutdown,
    },
    Signature {
        provider: "EventLog",
        ids: &[6008],
        kind: Kind::UnexpectedShutdown,
    },
    Signature {
        provider: "BugCheck",
        ids: &[1001],
        kind: Kind::BugCheck,
    },
    Signature {
        provider: "Microsoft-Windows-WHEA-Logger",
        ids: &[17, 18, 19, 47],
        kind: Kind::HardwareError,
    },
    Signature {
        provider: "Application Hang",
        ids: &[1002],
        kind: Kind::AppHang,
    },
    Signature {
        provider: "Application Error",
        ids: &[1000],
        kind: Kind::AppCrash,
    },
    Signature {
        provider: "Service Control Manager",
        ids: &[7000, 7001, 7011, 7031, 7034],
        kind: Kind::ServiceFailure,
    },
    Signature {
        provider: "disk",
        ids: &[7, 11, 51, 153],
        kind: Kind::DiskError,
    },
    Signature {
        provider: "Ntfs",
        ids: &[55, 98, 140],
        kind: Kind::Ntfs,
    },
    Signature {
        provider: "Display",
        ids: &[4101],
        kind: Kind::DisplayTdr,
    },
    Signature {
        provider: "nvlddmkm",
        ids: &[13, 14],
        kind: Kind::DisplayTdr,
    },
    Signature {
        provider: "Microsoft-Windows-Kernel-Processor-Power",
        ids: &[37],
        kind: Kind::ProcessorPower,
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Incident {
    /// `{channel}:{recordId}` — a record id is unique within its channel, not across the machine.
    pub id: String,
    pub time: String,
    pub kind: Kind,
    /// The first line of what the machine itself said, not a phrase this app made up.
    pub headline: String,
    pub event: EventRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub incident: Incident,
    pub from: String,
    pub to: String,
    pub events: Vec<EventRecord>,
    /// Exactly what the assistant would be given, so the page can show it before anything is sent.
    pub prompt: String,
}

pub fn classify(event: &EventRecord) -> Option<Kind> {
    SIGNATURES
        .iter()
        .find(|signature| {
            signature.provider.eq_ignore_ascii_case(&event.provider)
                && signature.ids.contains(&event.event_id)
        })
        .map(|signature| signature.kind)
}

/// The incidents in a run of events, newest first.
///
/// Two of a kind within a minute are one incident: a service that fails writes 7031 and 7034 in the
/// same breath, and listing both as separate incidents would make one fault look like two.
pub fn find_incidents(events: Vec<EventRecord>) -> Vec<Incident> {
    let mut candidates: Vec<(Kind, EventRecord)> = events
        .into_iter()
        .filter_map(|event| classify(&event).map(|kind| (kind, event)))
        .collect();
    candidates.sort_by(|left, right| right.1.time_created.cmp(&left.1.time_created));

    let mut incidents: Vec<Incident> = Vec::new();
    for (kind, event) in candidates {
        let collapsed = incidents.iter().any(|held| {
            held.kind == kind
                && seconds_between(&held.time, &event.time_created)
                    .is_some_and(|apart| apart.abs() <= COLLAPSE_SECONDS)
        });
        if collapsed {
            continue;
        }
        incidents.push(Incident {
            id: format!("{}:{}", event.channel, event.record_id),
            time: event.time_created.clone(),
            kind,
            headline: headline(&event),
            event,
        });
    }
    incidents
}

fn headline(event: &EventRecord) -> String {
    let first = event
        .message
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");
    if first.chars().count() <= 160 {
        return first.to_string();
    }
    let cut: String = first.chars().take(159).collect();
    format!("{cut}…")
}

/// The stretch of log around an incident: mostly before it, because that is where the cause is.
pub fn window(time: &str) -> AppResult<(String, String)> {
    let at = parse(time)?;
    Ok((
        stamp(at - Duration::minutes(BEFORE_MINUTES)),
        stamp(at + Duration::minutes(AFTER_MINUTES)),
    ))
}

fn parse(time: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(time)
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|error| AppError::Message(format!("{time} is not a timestamp: {error}")))
}

fn stamp(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn seconds_between(left: &str, right: &str) -> Option<i64> {
    Some((parse(left).ok()? - parse(right).ok()?).num_seconds())
}

/// One query for every signature at once, rather than one per provider.
pub fn incidents(days: u32) -> AppResult<Vec<Incident>> {
    let from = stamp(Utc::now() - Duration::days(i64::from(days.max(1))));
    let result = eventlog::query(&Filter {
        channels: vec!["System".into(), "Application".into()],
        levels: vec![1, 2, 3],
        from: Some(from),
        to: None,
        event_ids: SIGNATURES
            .iter()
            .flat_map(|signature| signature.ids.iter().copied())
            .collect(),
        providers: SIGNATURES
            .iter()
            .map(|signature| signature.provider.to_string())
            .collect(),
        max: SCAN_MAX,
    })?;

    Ok(find_incidents(result.events))
}

pub fn bundle(channel: &str, record_id: u64) -> AppResult<Bundle> {
    // The one event first, by its record id: that is the only way to select a single record, and
    // it is also what says whether it is still there.
    let parsed = eventlog::parse_event_xml(&eventlog::event_xml(channel, record_id)?)?;
    let kind = SIGNATURES
        .iter()
        .find(|signature| {
            signature.provider.eq_ignore_ascii_case(&parsed.provider)
                && signature.ids.contains(&parsed.event_id)
        })
        .map(|signature| signature.kind)
        .ok_or_else(|| {
            AppError::Message(format!(
                "{} event {} is not one of the incidents this page knows about",
                parsed.provider, parsed.event_id
            ))
        })?;

    let (from, to) = window(&parsed.time_created)?;
    let surrounding = eventlog::query(&Filter {
        channels: BUNDLE_CHANNELS
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        levels: vec![1, 2, 3],
        from: Some(from.clone()),
        to: Some(to.clone()),
        event_ids: Vec::new(),
        providers: Vec::new(),
        max: BUNDLE_MAX,
    })?;

    let events = without_noise(surrounding.events);
    let itself = events
        .iter()
        .find(|event| event.record_id == record_id && event.channel == channel)
        .cloned()
        .ok_or_else(|| AppError::Message(format!("{channel} no longer holds event {record_id}")))?;

    Ok(Bundle {
        prompt: assistant::render_events_for_prompt(&events),
        incident: Incident {
            id: format!("{channel}:{record_id}"),
            time: parsed.time_created,
            kind,
            headline: headline(&itself),
            event: itself,
        },
        from,
        to,
        events,
    })
}

pub fn without_noise(events: Vec<EventRecord>) -> Vec<EventRecord> {
    events
        .into_iter()
        .filter(|event| {
            !NOISE
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&event.provider))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(provider: &str, id: u32, time: &str, message: &str) -> EventRecord {
        EventRecord {
            record_id: u64::from(id),
            channel: "System".into(),
            provider: provider.into(),
            event_id: id,
            level: 2,
            level_name: "Error".into(),
            task: "None".into(),
            keywords: Vec::new(),
            time_created: time.into(),
            computer: "WORKBENCH".into(),
            message: message.into(),
            event_data: Vec::new(),
        }
    }

    #[test]
    fn every_signature_is_recognised_by_its_provider_and_id() {
        for signature in SIGNATURES {
            for id in signature.ids {
                let recognised = classify(&event(
                    signature.provider,
                    *id,
                    "2026-08-20T09:00:00.000Z",
                    "",
                ));
                assert_eq!(
                    recognised,
                    Some(signature.kind),
                    "{} {id}",
                    signature.provider
                );
            }
        }
    }

    /// The right id under the wrong provider is a different event entirely — 1000 is an application
    /// crash from Application Error and something else from everybody else.
    #[test]
    fn an_id_under_the_wrong_provider_is_not_an_incident() {
        assert!(classify(&event(
            "Contoso-Agent",
            1000,
            "2026-08-20T09:00:00.000Z",
            ""
        ))
        .is_none());
        assert!(classify(&event("EventLog", 6013, "2026-08-20T09:00:00.000Z", "")).is_none());
    }

    /// Every machine writes thousands of DCOM 10016s and none of them explains a crash.
    #[test]
    fn the_permission_noise_is_neither_an_incident_nor_part_of_a_bundle() {
        assert!(classify(&event("DCOM", 10016, "2026-08-20T09:00:00.000Z", "")).is_none());

        let kept = without_noise(vec![
            event("DCOM", 10016, "2026-08-20T09:00:00.000Z", ""),
            event("EventLog", 6008, "2026-08-20T09:00:01.000Z", ""),
        ]);

        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].provider, "EventLog");
    }

    #[test]
    fn incidents_come_back_newest_first() {
        let found = find_incidents(vec![
            event("EventLog", 6008, "2026-08-18T09:00:00.000Z", "older"),
            event("BugCheck", 1001, "2026-08-20T09:00:00.000Z", "newer"),
        ]);

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].headline, "newer");
        assert_eq!(found[1].headline, "older");
    }

    /// A service that fails writes 7031 and 7034 in the same breath. Listing both would make one
    /// fault look like two.
    #[test]
    fn two_of_a_kind_within_a_minute_are_one_incident() {
        let found = find_incidents(vec![
            event(
                "Service Control Manager",
                7031,
                "2026-08-20T09:00:00.000Z",
                "a",
            ),
            event(
                "Service Control Manager",
                7034,
                "2026-08-20T09:00:30.000Z",
                "b",
            ),
            event(
                "Service Control Manager",
                7031,
                "2026-08-20T09:05:00.000Z",
                "c",
            ),
        ]);

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].headline, "c");
        assert_eq!(found[1].headline, "b");
    }

    #[test]
    fn two_different_kinds_at_the_same_instant_are_both_kept() {
        let found = find_incidents(vec![
            event("EventLog", 6008, "2026-08-20T09:00:00.000Z", "shutdown"),
            event("BugCheck", 1001, "2026-08-20T09:00:05.000Z", "bugcheck"),
        ]);

        assert_eq!(found.len(), 2);
    }

    #[test]
    fn the_window_reaches_further_back_than_forward() {
        let (from, to) = window("2026-08-20T09:00:00.000Z").expect("a window");

        assert_eq!(from, "2026-08-20T08:45:00.000Z");
        assert_eq!(to, "2026-08-20T09:02:00.000Z");
    }

    /// Midnight is where naive arithmetic on the text of a timestamp goes wrong.
    #[test]
    fn a_window_crosses_a_day_boundary_without_help() {
        let (from, to) = window("2026-08-20T00:05:00.000Z").expect("a window");

        assert_eq!(from, "2026-08-19T23:50:00.000Z");
        assert_eq!(to, "2026-08-20T00:07:00.000Z");
    }

    #[test]
    fn a_headline_is_the_first_line_the_machine_wrote() {
        let found = find_incidents(vec![event(
            "EventLog",
            6008,
            "2026-08-20T09:00:00.000Z",
            "The previous system shutdown was unexpected.\nMore detail nobody needs in a list.",
        )]);

        assert_eq!(
            found[0].headline,
            "The previous system shutdown was unexpected."
        );
    }
}

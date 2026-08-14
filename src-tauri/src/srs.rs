use chrono::{DateTime, NaiveDateTime, Utc};
use rs_fsrs::{Card, Parameters, Rating, State, FSRS};

use crate::error::{AppError, AppResult};

/// The shape `datetime('now')` writes. FSRS has to read and write the same one, or
/// `due_at <= datetime('now')` stops comparing dates and starts comparing strings that only look
/// like dates.
const STAMP: &str = "%Y-%m-%d %H:%M:%S";

pub fn parse(stamp: &str) -> AppResult<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(stamp, STAMP)
        .map(|naive| naive.and_utc())
        .map_err(|error| AppError::Message(format!("timestamp {stamp:?}: {error}")))
}

pub fn stamp(at: DateTime<Utc>) -> String {
    at.format(STAMP).to_string()
}

pub const fn state_code(state: State) -> i64 {
    state as i64
}

pub const fn state_from(code: i64) -> State {
    match code {
        1 => State::Learning,
        2 => State::Review,
        3 => State::Relearning,
        _ => State::New,
    }
}

/// FSRS with its short-term steps off and its fuzz off.
///
/// Short-term makes a fresh card due again in minutes, which is a flashcard idiom: here it would
/// hand the question straight back inside the same sitting and collide with the Weak pool, which
/// clears only on two correct answers in *different* sessions. Long-term produces whole-day
/// intervals, the only unit `due_at` stores. Fuzz would put a random offset on every interval and
/// make two runs of the same session disagree.
fn scheduler() -> FSRS {
    FSRS::new(Parameters {
        enable_short_term: false,
        enable_fuzz: false,
        ..Parameters::default()
    })
}

/// A multiple-choice drill observes right or wrong and nothing else, so only two of the four FSRS
/// ratings can ever be produced — Hard and Easy would need a self-assessment the app never asks for.
const fn rating(correct: bool) -> Rating {
    if correct {
        Rating::Good
    } else {
        Rating::Again
    }
}

pub fn review(card: Card, now: DateTime<Utc>, correct: bool) -> Card {
    scheduler().next(card, now, rating(correct)).card
}

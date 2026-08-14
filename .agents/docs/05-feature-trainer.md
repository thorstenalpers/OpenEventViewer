# Trainer

## Session modes

| Mode        | Question set                                             | Scored | Explanations |
| ----------- | -------------------------------------------------------- | ------ | ------------ |
| Practice    | whole binder, shuffled                                   | yes    | after each   |
| Focus       | every question missed in a chosen earlier session        | yes    | after each   |
| Due         | everything the scheduler says is due today               | yes    | after each   |
| Weak topics | questions in topics below a score threshold              | yes    | after each   |
| Exam        | fixed count, fixed time limit, no feedback until the end | yes    | at the end   |
| Challenge   | Exam plus a fixed seed and a published rule set          | yes    | at the end   |

`needs_source` questions (image-based, figure missing — see
[03-ingest-pipeline.md](03-ingest-pipeline.md) §4) are excluded from every scored mode. They appear
only in Review.

## Wrong answers become the next session

This is the feature the product exists for, so it is not a filter buried in a dropdown.

When a session ends, the summary screen leads with the miss count and a single primary action:
**Start focus session (n questions)**. That session is created from the `attempts` rows of the
session just finished — not from a mutable "wrong list" that drifts out of date. A question leaves
the focus set by being answered correctly twice in a row across two different sessions, which
stops a lucky guess from clearing it.

Focus sessions are themselves recorded, so a focus session can spawn a focus session. The chain is
visible in Statistics as a drill-down.

## Scheduling

FSRS per question, stored in `scheduling` — the `rs-fsrs` crate with its default weights, driven
from `src-tauri/src/srs.rs`. Chosen over SM-2 because these question banks contain near-duplicate
questions, and interval growth needs to react to difficulty rather than to answer count alone.

A multiple-choice drill observes only right or wrong, so of the four FSRS ratings only two are ever
produced: correct is `Good`, wrong is `Again`. Hard and Easy would need a self-assessment the app
never asks for.

Two settings are off. **Short-term steps** would make a fresh question due again in minutes, which
collides with the focus pool — that pool clears only on two correct answers in _different_
sessions, and a question handed back inside the same sitting cannot honour it. **Fuzz** would put
a random offset on every interval, and two runs of the same challenge are supposed to agree.

- correct → stability grows by how much was recalled after how long, and the interval follows from
  it; a question answered right after three months moves further than the same question answered
  right the same afternoon
- wrong → stability drops, difficulty rises, lapse counter increments, question enters the focus
  pool. A first miss is not a lapse: FSRS only counts one against a question already in review,
  and you cannot forget something you never got right
- reps are not reset by a miss — the count is a history, not a position in a ladder
- a question with three lapses is surfaced in the Assistant view with "generate variants", because
  repeated failure on the same wording usually means the wording was memorised and the concept
  was not

Scheduling is per user profile and syncs when an account exists. It never leaves the machine
without one.

## Answer evaluation

- `SingleChoice` — one selection, exact match
- `MultipleChoice` — the required selection count is shown ("choose 2"), and partial credit is
  **not** given; the real exams do not give it either
- `Matrix` — every box must match; per-box feedback is shown afterwards
- Elapsed time per question is recorded on every attempt, in every mode, because challenges and
  speed statistics both read from `attempts` and neither should need its own capture path

## Timing and challenges

A challenge is `{ binder_version, seed, question_count, time_limit, show_explanations }`. The seed
fixes the order, so two people take the identical exam. Results record per-question elapsed time,
which is what makes "who solved it fastest" comparable rather than a function of who got the
easier shuffle.

Local challenges need no account. Publishing one, or appearing on its leaderboard, does.

## Statistics

Derived from `attempts` only. Per binder, per topic, per question: accuracy, median time,
attempt count, lapse count, trend over time. TanStack Table with column sorting and faceted
filters, the same table shell as Library and Catalog.

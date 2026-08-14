# Roadmap

Ordered so that every milestone ends with something usable. Nothing later blocks anything earlier.

## M1 — Shell

Tauri 2 + SvelteKit + shadcn-svelte + Tailwind v4 scaffold, sidebar, routing, theme, settings
store, the bridge with its mock host, CI (fmt, clippy, lint, check, vitest, cargo test).

Done when `npm run dev` shows the sidebar and every route renders an empty state, and
`npm run start` shows the same thing in the Tauri window.

**Status: done.** Sidebar, eight routes, light/dark, the Zod contract with its mock host, the SQLite
schema and ~30 host commands. `npm run lint`, `npm run check`, `npm test` (31), `cargo fmt`,
`cargo clippy -- -D warnings` and `cargo test` (43 host, 37 ingest, 6 fixture) all pass.

**CI is `.github/workflows/ci.yml`**, Windows only — WebView2, the Credential Manager, Windows
speech synthesis and win-x64 Pdfium are the product, so a green tick on another platform would mean
nothing. A `check` job runs every check above; a `bundle` job regenerates the licence notices and
builds the NSIS installer, uploaded as an artifact, skipped on pull requests. Only first-party
actions, so nothing third-party executes in the run. Pdfium is fetched rather than committed and
cached so the unauthenticated GitHub API call is rare.

Two things CI does **not** prove, both by design. The ingest fixture tests skip themselves when the
vendor exam PDFs are absent — that material is gitignored and never redistributed — so those tests
reporting ok in CI means they did not run. And `the_local_binary_answers` stays `#[ignore]`d,
because it spends a real request.

One thing unknown until it runs: `an_episode_is_synthesised_end_to_end` drives Windows speech
synthesis, which needs an installed voice. Whether a GitHub Windows runner has one has not been
established here.

`tauri build` runs locally and produces the NSIS installer. What that does _not_ prove is the thing
the bundle exists for: the installer has not been installed and the installed app has not been
started, so "the app runs as a package with `pdfium.dll` beside it" is still an untested claim. The
app has only been run through `tauri dev`, out of `target/debug`.

Not in it, on purpose: the theme is browser-local rather than round-tripped through
`get_settings` / `set_settings`, which exist on both sides but are not wired to a view yet.

## M2 — Import and Review

`ingest/pdf.rs`, `layout.rs`, `parse.rs`, the `certshared` / `certleader` / `generic` profiles,
confidence scoring, the Review view.

Done when the two sample PDFs import to 8 and 11 questions with correct stems, options and answer
keys, the five `Mastered/Not Mastered` questions in each are either given their figure back or
flagged `needs_source` rather than presented as answerable, the trailing paywall marker is reported
as a stub rather than a question, and no extracted string contains footer text. Fixture tests over
both files are the gate.

**Status: done.** `crates/ingest`, 25 tests, `cargo fmt` and `cargo clippy -- -D warnings` clean.
The Review view itself belongs to M1's shell and lands with it.

**Figures recovered.** All ten `Mastered/Not Mastered` questions across the two files get their
answer area back, so none is `needs_source` any more and all of them are drillable. See
[03-ingest-pipeline §4](03-ingest-pipeline.md) — including the correction to this document's own
earlier claim that these PDFs contained no images.

## M3 — Trainer

Session engine, the six modes, FSRS scheduling, `attempts`, the summary screen with **Start focus
session** as its primary action.

Done when a practice run on the AI-900 binder produces a focus session containing exactly the
questions missed, and a question answered correctly twice across two sessions leaves the pool.

**Status: done.** Practice, Focus and Due run; `attempts` is append-only and the focus set is read
back out of it; the summary screen leads with **Start focus session**.

**Now also done:** Exam and Challenge (M10), and a persistent **Weak** pool — a question enters it
by being missed and leaves it only after two correct answers in a row _in different sessions_, so a
lucky guess and a repeat inside one sitting both fail to clear it.

**Scheduling is FSRS**, not SM-2. The `rs-fsrs` crate with its default weights, short-term steps
and fuzz both off; a binary drill produces only `Good` and `Again`. Nothing in the old table
converted — `ease` and FSRS difficulty are different quantities — so the migration drops it and
**replays every row of `attempts` through the scheduler** at the timestamps those attempts carry.
That is only safe because `attempts` is append-only, which it is, and a test asserts the replay
lands on the state the same reviews produce live.

## M4 — Binder

`.examdeck` read/write, asset store, links from extracted `Reference:` URLs, notes, the Library
table.

Done when a binder exports, the export imports on a clean profile, and the round trip is
byte-identical for questions and assets.

**Status: done for questions, links, videos, notes and figures.** `.examdeck` is a zip of five JSON
files plus a `figures/` folder; attachments reference a question by its **position**, not its
database id — ids are machine-local and question numbers repeat in these dumps, so position is the
only anchor that survives the trip. Figures are content-addressed, so the folder is keyed by the
same SHA-256 the questions carry and unpacking is idempotent. Recorded audio is not in the zip.

## M5 — Browse and Media

Site webview with a persistent profile, bookmarks from binder links, video entries with
per-question anchors and start timestamps.

Done when a Microsoft Learn login survives an app restart and a question's video anchor opens at
its timestamp.

**Status: built, one claim unverified.** The window is now a bare `Window` with two child webviews;
the site webview is created on first navigation, parked off-screen when Browse is left, and placed
from a rectangle the Browse view measures, so the sidebar width is never duplicated in Rust. Only
the `chrome` webview is listed in the capability, so a foreign page cannot reach a command.

Not verified: that a Microsoft Learn login survives a restart — that needs a human at the window.

## M6 — Assistant

Lifted from CleanMyPosts: local `claude` binary as the default source, hosted providers opt-in,
key in the Windows Credential Manager. Question explanation, variant generation, note
summarisation, link suggestion.

Done when the assistant answers about the current question without any key configured.

**Status: the local path is verified, the hosted one is not.** Local `claude` binary by default —
probed directly first, then through `cmd /C`, because on some machines `claude` is a `.cmd` shim
that `Command::new` will not execute and on others it is a plain `.exe`. Anthropic API opt-in with
the key in the Credential Manager; that path handles `stop_reason: refusal` and opts into
server-side fallbacks.

`the_local_binary_answers` runs the real binary and reads a real reply back, so the transport is no
longer an assumption. It carries `#[ignore]`: it needs the binary installed and it spends a
request, neither of which belongs in a check that runs on every commit.

Still not verified: the hosted path. It needs a key in the Credential Manager and there is none on
this machine — putting one there is the user's step, not the agent's.

## M7 — Podcast

Script builder (question / pause / answer / explanation, each toggleable), Windows speech
synthesis driver, optional hosted TTS driver, chapter marks, export.

Done when a topic selection produces a playable file offline, with no key and no network.

**Status: done.** Windows speech synthesis writes one WAV per segment at a pinned 22 050 Hz / 16 bit /
mono; Rust concatenates them with real silence and derives chapter offsets from the byte counts. The
format is checked rather than assumed — a mismatched sample rate would play at the wrong pitch with
no error anywhere. Verified by a test that synthesises a real episode.

**MP3 is now the default export**, WAV the alternative. The encoder is `rusty_mp3` — pure Rust,
Apache-2.0 — rather than LAME: the LAME bindings are LGPL-3.0 over an LGPL-2.1 library, and static
linking one into an MIT installer brings a relinking obligation that is not worth a codec. 22 050 Hz
puts the stream on MPEG-2 at 64 kbps.

Measured on a 55.6 s synthesised episode, not estimated: 5.5× smaller than the WAV, 64 kbps actual,
34.2 dB SNR overall and 33.3 dB segmental over 1408 voiced windows, and band energy within ±0.3 dB
of the source at every frequency from 200 Hz to 10 kHz — no audible bandwidth was traded away. The
`mp3_against_the_wav_it_came_from` diagnostic prints those figures on demand. Two caveats it cannot
address: it decodes with the same crate that encoded, so a shared misreading of the format would
cancel out, and nobody has listened to the result.

Chapter offsets are not recomputed for MP3: they are times, and the encoder rearranges bytes rather
than the clock. The written file is checked the same way the WAV path is — a test encodes a known
tone and decodes it back, and the end-to-end test now writes both formats from one synthesised
episode and asserts the chapters come out identical.

**The voice is English or German**, chosen rather than inherited from the system default. The
choice picks the speech voice _and_ the words wrapped around each question — "Question 3." against
"Frage 3.", "The answer is B." against "Die richtige Antwort ist B.". The question itself is never
translated: it is the user's imported material, so German narration over an English bank reads
English sentences in a German voice, which is allowed and rarely wanted.

A missing voice is an error naming the ones that are installed, not a silent fallback to whatever
the system had. Reading German text with an English voice is not an accent, it is a different set
of letter sounds. Verified on a machine with only `Microsoft David` and `Zira` (both `en-US`):
English selects David, German exits with `no 'de' speech voice is installed. Add one in Windows
Settings under Time and language, Speech. Installed: …`.

**Downloadable neural voices** answer that error rather than only reporting it. Settings offers a
catalogue of Kokoro packs — German (Martin), German (Kerstin) and English — each fetched on a button
press into `app_local_data_dir/voices`, run through sherpa-onnx in the host. The list is read from
that folder at every start and again after every attempt, so a pack deleted between two runs stops
being offered and a pack copied in by hand counts. A chosen pack reads the episode instead of
Windows; the language choice above then only picks the words wrapped around each question.

Both German packs exist on Hugging Face as community exports rather than as sherpa archives, so
`hub.rs` assembles them: espeak's data comes from the sherpa mirror, the voice table is flattened
where it is a NumPy archive, and the metadata sherpa reads is appended to the model. The token table
is the mirror's only for a model that states no vocabulary of its own — the two Kokoro generations
differ by eleven entries, `A` among them, which is enough to mispronounce a word, so Kerstin's
comes out of its own `tokenizer.json`. Downloads report progress, can be cancelled, and are held in
memory until complete: a half-written folder that looks installed is worse than holding a few
hundred megabytes. The unpack that follows is minutes long and says so, because a bar sitting at
100 per cent reads as a download that died.

**Verified end to end** on 13 August 2026: all three packs fetched, assembled and spoken. English
2.2 s at 24 kHz, Martin 9.6 s, Kerstin 11.4 s, each with a real peak rather than silence. espeak
hands Kokoro a combining cedilla it has no token for and sherpa skips it, in both German packs —
audible only as a missing diacritic. `voice::probe::a_pack_installs` and `a_pack_says_something`
are the `#[ignore]`d tests that repeat this; `WAV=…` on the second writes the sample out.

**The preview was slow, and the measurements said why.** On eight cores, one 1.8 s sentence took
2.7 s at one thread, 1.6 s at two, 1.4 s at four and 0.94 s at eight — so the two threads it started
with were leaving most of the machine idle. Threads now come from `available_parallelism`, capped
at eight. The model load costs a further 2 s and is unaffected by threads, so it is paid when a
voice is chosen rather than when play is pressed. And playback itself waited `2 × length`: it
required `sink.get_pos() >= length`, which never becomes true because the position resets when the
queue drains, so every preview fell through to the escape hatch. Waiting on `sink.empty()` and then
600 ms for the device buffer took one sentence from 8.6 s end to end to 3.9 s, and from 4.1 s to
1.4 s before the first sound. `what_a_preview_costs_end_to_end` and
`where_the_wait_before_a_preview_goes` print these figures on demand; both make a noise.

The thread change is worth more to an episode than to a preview: a podcast is hundreds of segments,
each one synthesised in turn.

**The hosted TTS driver is dropped**, not pending. It was in the milestone as the answer to "the
Windows voices are not good enough", and the downloadable packs answer that without an account, a key
or a request leaving the machine — which is the better answer by
[hard rule 1](../../AGENTS.md), not merely an equivalent one. Building it anyway would mean shipping
a network client nobody here can run: verifying it needs a key in the Credential Manager, and code
that has never once done its job is the thing this project refuses elsewhere. If a voice arrives that
no pack can match, this is a day's work and the decision can be revisited.

## M8 — VCE probe

Header probe, version identification, decoder dispatch, the honest failure path with the
PDF-export alternative. See [adr/0002](adr/0002-vce-support-scope.md).

Done when both `.vce` samples report a specific unsupported-version message naming their
signature, and the support matrix in [03-ingest-pipeline.md](03-ingest-pipeline.md) §8 reflects
reality.

**Status: done.** No decoder exists and none is claimed.

## M9 — Accounts and sync

Progress sync and per-machine reconciliation.

Done when progress made offline on one machine appears on another after login.

**Status: the sync is built, against a file rather than a server.** The backend is a second SQLite
database and a folder of decks in the app's own data directory — `catalog.sqlite3`, written by
`src-tauri/src/catalog.rs`. Everything above that module is the real thing; that module is the part a
server replaces, and which server is undecided. The schema is ordinary relational SQL with no
extension in it, so it does not force the answer.

**A question is paired across machines by its content.** `question_key` is the SHA-256 of the
whitespace-collapsed stem and the sorted answer key, because row ids are per-machine and question
numbers repeat inside a single dump. A test imports the same question into two libraries at
different row ids and shows the key still pairs them.

**Only the schedule travels.** `attempts` is an aggregate on the wire and append-only locally, so
filling the local log to make two counts agree would be forging a history. Push refuses to overwrite
a stored row that holds more attempts than the local one — append-only means a lower count is an
older state whatever its clock says — and pull compares `last_review_at`, so running it twice reports
nothing the second time.

**No accounts**, which is the one thing a server would have to bring. `identity` is a single row
drawn on first open, and every ownership check this module makes happens in Rust — a check the client
performs is a check the client can skip, so it is a stand-in and not a security boundary. On one
machine the sync is a round trip to itself: what it proves is the pairing, not the network.

## M10 — Catalog and challenges

Publish with an upload preview, catalog browse with the filtering and sorting done in SQL, ratings,
challenge rule sets, leaderboards.

Done when a binder published from one profile is importable and rateable from another, and two
runs of the same challenge produce identical question order.

**Status: done, against M9's file.** Exam and Challenge modes withhold feedback until the end and run
under a clock that finishes the session when it expires. A seed draws the question order in Rust
(xorshift64* over a number-ordered list) rather than with SQL `RANDOM()`, which cannot be seeded —
so the same seed is the same exam on any machine. The leaderboard ranks by score, then by time.

**Publishing shows what would be published first**, and the figures come off the deck the publish
actually writes rather than off a second count of the tables — a preview assembled another way is a
preview of something else. Publishing again replaces the entry it already has, because `binders`
carries the entry id it was published as; withdrawing takes the deck file with it and clears that id,
so the next publish is a new entry rather than the resurrection of one somebody may have rated.

**Ratings are one row per person per binder**, and the aggregate the catalog sorts on is kept by
triggers rather than by the author — an author who could write those two numbers could invent their
own score. Filtering and sorting run in SQL, not in the view, which is the division a server-backed
catalog needs.

**A finished challenge reaches the board from the Train summary**, and only when the binder has been
published — the board belongs to a catalog entry, so a binder without one is told why the offer is
missing rather than shown a button that would fail. An unseeded run is refused rather than posted
under seed 0.

Everything here is local: the catalog is a file on this machine, so publishing shares a binder with
nobody yet.

**Tested on three levels.** `catalog.rs` in memory, for the rules — the aggregate, the ownership
check, the seed board, the sync merge. The publish path on disk, for what only files can show: two
databases opened from a directory, a deck written into the catalog folder, and the byte count the
entry reports taken off that file rather than from a second count of the tables. And the views
against the mock host, for the preview, the ownership rules and the posting path.

What none of the three touches is the Tauri host: the `#[tauri::command]` wrappers are now thin
enough that what they add over the tested functions is the `State` lookup and a log line, but
`catalog.sqlite3` has still never been opened by the running app.

## M11 — Language and colours

English and German interface, a colour-preset picker, and both remembered across restarts.

Done when every string a view paints comes from `src/lib/i18n/`, a test proves the two
dictionaries have the same keys, and the chosen palette is on `<html>` before the first frame.

**Status: done.** Six palettes from the tweakcn registry (the vendor-branded ones were stripped);
light/dark stays a separate axis, so a preset and `.dark` compose. `<ModeWatcher>` runs with
`synchronousModeChanges`: its default defers the swap into a `requestAnimationFrame`, and a webview
that is not compositing never fires one — minimised or occluded, the theme would move in
`localStorage` and not on screen until the next load. Preferences live in
`localStorage` rather than the host's settings file — they only change what this webview paints,
and reading them synchronously is what stops the app flashing the wrong palette on every start.

Question text is never translated: it is the user's imported material, and a translated exam
question is a different question.

## M12 — Statistics and notes

Per-question and per-topic accuracy, timing and lapses; notes attached to a question, including an
assistant answer kept as one.

Done when every number in the view is derived from `attempts` and `scheduling` rather than from
a counter, and a note written in Review and a note saved from the assistant land in the same list.

**Status: done.** An unanswered question reports _no_ accuracy rather than 0 % — the difference
between 'never tried' and 'always wrong' is the whole point of the view. The topic roll-up is
computed from the same rows the table shows, so the two cannot disagree.

## M13 — The exam as a project

An exam is more than a question bank: it is started on a date, worked through, and passed — more
than once, because a certification expires. This milestone gives that shape somewhere to live.

**The schema grew four ways.** `templates` holds an exam before anyone studies for it, keyed on name
_and_ documentation URL together: the same code under a different study guide is a different exam,
and the same guide under two names is one exam typed twice. `certifications` is one row per pass, so
a renewal is history rather than an overwrite. `progress` is one row per ticked step, so an untouched
step stores nothing. `links` gained a description, a kind and a duration; `binders` gained the
documentation URL. New tables arrive through `CREATE TABLE IF NOT EXISTS`; new columns through a
checked `ALTER TABLE`, because the first does nothing for a table that already exists.

**The catalogue is seeded once** with fifteen Microsoft certifications, and only into an empty table
— a template the user deleted stays deleted. Every URL follows Microsoft's own
`study-guides/<code>` pattern and every one of them answered 200 on 14 August 2026.

**The overview has a time axis.** One row per exam, from the day it was created to the last thing
that happened to it, with a marker for every pass. Year ticks are drawn once above all the rows so
the dates line up by position rather than by everyone reading their own scale.

**The exam page** opens from the project name and holds the summary, the pass dates and a checklist
that stays on screen while the rest scrolls. Three of its six steps are derived rather than stored —
the project exists, questions have been answered, a pass date is on file — because asking someone to
tick a box the app already knows the answer to is asking them to keep two records of one fact.
Ticking a stored step plays a short animation, which `prefers-reduced-motion` turns off.

**Study is now its own page**, separate from Train: courses, videos and documentation with their kind
and duration, filtered by a row of shelves that carry their own counts. A video off the user's own
disk plays in the app through Tauri's asset protocol — the webview refuses `file:` URLs, so the
`protocol-asset` feature and an `assetProtocol` scope are what make it reachable.

**Notes became a workshop.** The assistant rewrites them as a Markdown summary stored beside the
project, and that summary can be read out as an episode by the same synthesiser the question podcast
uses — headings become chapters, because that is where the subject changes. Both artefacts are files
with a delete button. An artefact is addressed by name and never by path, so the webview cannot ask
for a file elsewhere on the machine by dressing it up as one of these.

**PDF export is built.** `pdfium` reads PDFs and cannot write one, and the library that would fill
the gap brings seventy-odd crates for a single button — so `typeset.rs` writes the file: a text-only
PDF over a built-in font is a page of structure. One typeface at three sizes, because wrapping needs
the advance of every glyph it sets and a second set of metrics is a second table to get wrong.

The claim a writer cannot make about itself is made by the reader instead. `pdfium` is already here,
so a test sets a German summary, opens the result with it, and asserts the umlauts and the em dash
came through WinAnsi and that no line's right edge passes the margin. What no test can say is
whether it looks good: nobody has opened one in a viewer.

**Verified in the mock host**, not in the app: the timeline draws all three fixtures with their pass
markers, the checklist ticks and derives, the study shelves count and filter, and a note becomes a
summary and then an episode. What the mock cannot show is the assistant actually writing a summary,
sherpa reading it, or a local video playing — those need the Tauri host.

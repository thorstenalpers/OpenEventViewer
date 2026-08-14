# Backend — a file today, a server later

The catalog runs. It is not Supabase: it is a second SQLite database and a folder of decks in the
app's own data directory, written by [`src-tauri/src/catalog.rs`](../../src-tauri/src/catalog.rs).
Publishing, browsing, importing, rating, leaderboards and progress sync all work against it, with no
project to create and no key to store.

`supabase/migrations/0001_catalog.sql` is still the Postgres shape of the same thing, and still has
never run. The two are kept column for column on purpose — see [Why a file](#why-a-file).

The desktop app is complete without either. Import, review, drill, statistics, export, browse and
podcast work with no account and no network — that is [hard rule 1](../../AGENTS.md). The catalog
adds sharing and cross-machine sync and nothing else.

## What exists

| Piece        | Where                                                       |
| ------------ | ----------------------------------------------------------- |
| The database | `app_local_data_dir/catalog.sqlite3`                        |
| The decks    | `app_local_data_dir/catalog/<entry-id>.examdeck`            |
| The module   | `src-tauri/src/catalog.rs`                                  |
| The commands | `catalog_*` and `progress_*` in `src-tauri/src/commands.rs` |
| The view     | `src/lib/views/catalog-view.svelte`, route `/catalog`       |

`identity` is one row, drawn on first open, standing in for `auth.users`. It is what `owner_id`,
`rater_id` and `runner_id` point at, and it carries a display name the Catalog view can change.
There is no password, because there is nobody to prove anything to yet.

## Why a file

Writing a network client against a project that does not exist produces code that compiles, ships,
and has never once done its job. Writing it against a local database that has the same tables, the
same keys and the same constraints produces code that has done its job several thousand times before
anybody points it at a server.

So the split is: everything above `catalog.rs` — the commands, the contract, the view, the upload
preview, the question key, the merge rule — is the real thing and does not change. `catalog.rs` is
the part that gets replaced.

The orchestration is kept out of the `#[tauri::command]` wrappers for the same reason `import_into`
is: `preview_publication`, `publish_into`, `withdraw_from` and `import_entry` take connections and a
path, so a test can run the whole publish against real files. The wrappers add a `State` lookup and a
log line and nothing else.

What the file cannot stand in for, and what will therefore still be unproven on the day a project
exists: the transport. Latency, partial writes, an expired token, two machines writing at once. A
`Mutex` around a SQLite connection settles all four for free, and a network settles none of them.

## Why the schema looks like this

**The catalog stores manifests, not decks.** `published_binders` carries the fields the catalog
sorts and filters on — title, certification, question count, profile — plus a `storage_path`. The
`.examdeck` itself sits in the catalog folder. Filtering and sorting happen in SQL rather than in the
view, which is the division a Postgres-backed catalog needs, so the table above it does not move when
one arrives. The download is the same zip [M4](14-roadmap.md) already produces, which is what keeps
publish, share and import one code path.

`storage_path` is a file name, never an absolute path: a data directory moves with the profile, and a
stored path would point at the old machine's disk.

**Republishing replaces.** `binders.remote_id` records which entry a binder was published as, so
publishing again rewrites that entry rather than adding a copy. `published_at` stays put and
`updated_at` moves — the catalog sorts by the day a binder first appeared, and a typo fix should not
send it back to the top. Withdrawing clears `remote_id`, so the next publish is a new entry rather
than the resurrection of one somebody may already have rated.

**Ratings are keyed `(binder_id, rater_id)`.** One row per person per binder. Without that key a
rating is a vote counter, and the first person with a script owns the leaderboard.

**The rating aggregate is maintained by three triggers, not by the author.** `rating_count` and
`rating_sum` are denormalised onto the binder so the catalog can sort without a join — but an author
who could write those columns could invent their own score. In Postgres the update policy grants the
row and the trigger owns the two numbers; here the triggers own them and nothing in `catalog.rs`
writes them directly.

**Challenge results are never updated.** A posted time is a record. An editable record is not one. A
second run is a second row.

A result reaches a board from the Train summary, and only from a challenge run on a binder that has
been published — a board belongs to a catalog entry, so a binder with no entry is told why the offer
is missing rather than shown a button that would fail. `binders.remote_id` is what the summary reads
to know which board it is. An unseeded run is refused in `db::session_result` rather than posted
under seed 0: the board is per seed, and a run that shares its question order with nothing has
nothing to be compared against.

**Progress is keyed by question content, not by row id.** `question_key` is the SHA-256 of the
whitespace-collapsed stem and the sorted answer key. Local ids are per-machine, so syncing them would
pair the wrong rows on the second device; question numbers repeat inside a single dump, so those are
no better. A test imports the same question into two libraries at different row ids and shows the key
still pairs them.

**The whole FSRS card crosses, not a summary of it.** An earlier draft of the Postgres file carried
stability, difficulty and a due date. The scheduler also reads `reps`, `state`, `elapsed_days` and
`scheduled_days`, so a machine receiving those three would have to invent the other four. Both files
now carry all nine.

## What the sync does and does not do

**Push** sends every answered question up. A stored row with more attempts than the local one is left
alone: `attempts` is append-only, so a lower count is an older state whatever its clock says, which
makes this safe against two machines' clocks disagreeing where a plain last-write-wins is not.

**Pull** brings the card down onto whichever local questions match by key — and only the card.
`attempts` and `correct` are aggregates, and the local table they would have to land in is
append-only. Writing rows into it so the two counts agree would be forging a history, so the counts
stay local and only the schedule travels.

Which is why a repeated pull is judged on the card rather than on the counts. The local `attempts`
never rises to meet the stored one, so a pull comparing only those two would report the same row as
applied every time it ran. It compares `last_review_at` instead, and the second pull reports nothing.

On one machine this is a round trip to itself. What it proves is the pairing, not the network.

## What is deliberately not written

No auth, no tokens, no HTTP client. When there is a project to point at, the order is:

1. Auth, and `identity` becoming the account rather than a drawn id.
2. Publish: upload the zip to the `decks` bucket, insert the manifest row, keep the upload preview
   exactly as it is — it already shows what would leave the machine before it leaves
   ([hard rule 2](../../AGENTS.md)).
3. Catalog browse and import — read-only, so a bug costs nothing.
4. Ratings, whose triggers become the Postgres ones already written.
5. Progress sync, whose merge rule does not change.
6. Challenge leaderboards, which are these plus a real `runner_id`.

Steps 2 to 6 are written and tested. What each of them needs is step 1 and a different `catalog.rs`.

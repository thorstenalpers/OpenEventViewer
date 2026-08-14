# Architecture

## The window

Copied wholesale from CleanMyPosts, because it is the right shape for this app too. One Tauri
window containing nothing but child webviews (`tauri` with the `unstable` feature, for
`Window::add_child`):

- **chrome webview** — the SvelteKit app, prerendered by `adapter-static`, served by Tauri from
  the bundle. Sidebar, all views, all controls.
- **site webview** — an embedded browser. Microsoft Learn, vendor documentation, YouTube, any
  learning portal. Its own persistent WebView2 profile, so logins survive restarts. Shown only
  in Browse and Media; hidden and unloaded elsewhere.

No native UI toolkit. The chrome page is prerendered, so the sidebar paints before any script runs.

## Layers

Extraction is a workspace crate rather than a Tauri module, so its fixture tests run under plain
`cargo test` with no window, no WebView2 and no bundled frontend. `src-tauri` depends on it.

```
crates/ingest/     openexamtrainer-ingest — see 03-ingest-pipeline.md

src-tauri/src/
  main.rs
  lib.rs
  bridge.rs          typed RPC in, push events out — same two protocols as CleanMyPosts
  error.rs
  settings.rs        settings.json in app_config_dir
  state.rs
  db/                rusqlite, migrations, repositories
  trainer/           session engine, scheduler, scoring   — see 05-feature-trainer.md
  binder/            binder model, .examdeck read/write, asset store
  media/             video entries, podcast script builder, TTS drivers
  assistant/         cli.rs, providers.rs, secrets.rs     — lifted from CleanMyPosts
  catalog.rs         the second database: publish, browse, ratings, boards, progress sync
  commands/          one module per view's command surface

src/
  lib/bridge/        the typed client, and the mock host used by `npm run dev`
  lib/components/    bridge-free presentational components (props in, events out)
  lib/components/ui/ shadcn-svelte
  lib/stores/        Svelte 5 runes classes, one bridge subscription each
  lib/views/         the seam that talks to the bridge
  lib/schemas/       Zod — the single source of types and runtime validation
  routes/            library, import, review, train, browse, media, assistant,
                     catalog, stats, settings
```

## A project is a binder

The interface calls the unit of work a **project** — one certification, `AI-102`. Storage calls the
same row a **binder**, which is what it was called before projects existed.

They are the same thing, deliberately: a project holds exactly one imported file, and with it the
questions, links, videos and notes that hang off it. Modelling a project that _owns_ a binder would
have added a table, a foreign key and a join to express a one-to-one relationship — two names for
one row is the cheaper honesty, and this paragraph is the whole cost of it.

Two consequences worth knowing:

- A project can exist with no file yet. `create_project` writes the row with an empty `source_file`;
  a later import fills it. The projects table shows those as `no file yet` and offers **Add file**
  instead of **Train**, because training an empty project would open a session with nothing in it.
- A project takes **one** file. `fill_project` refuses a second import rather than appending, since
  merging two exams into one project would average two unrelated scores together.

## Storage

SQLite via `rusqlite` with the `bundled` feature, in `app_local_data_dir`. Project assets sit in a
content-addressed folder next to it, referenced by hash — so re-importing the same figure twice
costs nothing and exporting is a file copy.

Core tables:

```
binders        id, title, certification, source_kind, created, updated, remote_id
questions      id, binder_id, kind, stem, explanation, confidence, provenance, source_page
options        id, question_id, letter, text, is_correct
assets         hash, kind, path
question_assets question_id, hash, role
links          id, binder_id, question_id?, url, title, kind
videos         id, binder_id, question_id?, provider, ref, start_seconds, title
notes          id, binder_id, question_id?, body_md
attempts       id, question_id, session_id, given, correct, elapsed_ms, at
scheduling     question_id, due_at, last_review_at, stability, difficulty, elapsed_days, scheduled_days, reps, lapses, state
sessions       id, binder_id, mode, seed, started, finished, rule_set
```

`attempts` is append-only. Every statistic, every wrong-answer set and every challenge result is
derived from it, so nothing is ever double-booked.

## Contracts

Zod schemas in `src/lib/schemas/` are the source of truth for the bridge payloads. The Rust side
mirrors them with `serde` structs, and a `cargo test` walks the schema list to assert the two
sides agree on field names and required-ness. Drift is a test failure, not a runtime surprise.

## The catalog

`catalog.rs` and a second SQLite file: publish, browse, ratings, leaderboards and progress sync,
all against `catalog.sqlite3` and a folder of decks in the app's own data directory. It is a real
implementation rather than a placeholder, and on one machine it is a catalog of one person.

The boundary is where a server would go: the commands, the contract, the views, the upload preview,
the question key and the merge rule sit above it and do not know what is underneath. See
[16-backend.md](16-backend.md) for the schema and for what a networked one would have to add —
authentication, and everything that follows from two machines writing at once.

Nothing in the desktop app blocks on any of it. Import, review, drill, statistics, export, browse
and podcast are complete with the catalog untouched.

## Third-party notices

The customer gets a binary. MIT, BSD and ISC each require the copyright notice **and** the licence
text to accompany a binary distribution, and Apache-2.0 requires the licence itself — so a list of
SPDX identifiers is a summary, not compliance, and a link to a release page is not either.

`npm run licenses` (`scripts/create-licenses.mjs`) writes two artefacts:

- `src-tauri/resources/THIRD_PARTY_LICENSES.txt` — every component with its full licence text.
  Declared in `tauri.conf.json` under `bundle.resources`, so NSIS packs it and the installer writes
  it to `$INSTDIR\resources\`. That file being there is what meets the obligation.
- `src/lib/third-party.json` — name, version, licence and whether a text exists. The Info page
  renders it as a filterable list and fetches the full text on demand through
  `third_party_licenses`. Texts are kept out of the bundle: 596 of them would cost a megabyte of
  prerendered JavaScript for something read once.

Scope: Rust crates for `x86_64-pc-windows-msvc` with build- and dev-dependencies excluded; **all**
npm packages, because adapter-static bundles at build time and the dependencies/devDependencies
split says nothing about what ends up in the assets.

Two things in `vendor/pdfium/` that cost an hour to find, both wrong in the obvious reading:

1. `vendor/pdfium/LICENSE` is **not** PDFium's licence. The binaries come from
   bblanchon/pdfium-binaries and that file is the packaging repository's MIT licence. Shipping it
   under the heading "PDFium — BSD-3-Clause" would have attributed a stranger's copyright to the
   wrong licence. PDFium's own notice is `licenses/pdfium.txt`.
2. `licenses/pdfium.txt` carries **two** licences — BSD-3-Clause for PDFium and the full
   Apache-2.0 text for parts taken from elsewhere. A detector that returns the first match labels
   it `Apache-2.0` and silently drops the licence that governs the library, which is why `spdxOf`
   collects every licence a text announces instead of picking one.

`pdfium.dll` also statically contains freetype, icu, libjpeg-turbo, zlib and ten more. Their
notices are in `vendor/pdfium/licenses/` and ship whether or not anyone lists them.

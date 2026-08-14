# OpenExamTrainer

A Windows desktop app that imports certification exam material (PDF dumps, VCE files),
extracts the questions, drills them, and feeds every wrong answer back into a targeted
follow-up session. Around that core sit a study binder ("Lernmappe") with links, docs and
videos, an embedded browser for learning portals, an AI assistant, a podcast generator, and
a catalog for publishing, rating and challenging binders.

Desktop is the primary product and there is no server: everything, catalog included, is a file
in the app's own data directory. Nothing needs an account and nothing reaches the network
except a voice pack somebody asked for.

## Language

All documentation, code, comments, commit messages, and diagram labels are written in
**English**.

## Stack

| Layer     | Technology                                                                       |
| --------- | -------------------------------------------------------------------------------- |
| Host      | Tauri 2 (Rust) in `src-tauri/`, WebView2                                         |
| UI        | SvelteKit (prerendered, `adapter-static`), Svelte 5 runes, TypeScript            |
| UI kit    | shadcn-svelte (`new-york`, `neutral`), Tailwind v4, runtime colour presets       |
| Language  | English and German, `src/lib/i18n/`; question text is never translated           |
| Tables    | TanStack Table v8 (Svelte adapter) — catalog, question lists, statistics         |
| Contracts | Zod — one source for types and runtime validation                                |
| Storage   | SQLite (`rusqlite`, bundled) in `app_local_data_dir`; assets on disk beside it   |
| Ingest    | `pdfium-render` (text with coordinates, embedded images, page rendering)         |
| Speech    | Windows `System.Speech`, or a downloaded Kokoro pack through `sherpa-onnx`       |
| Catalog   | a second SQLite beside the library, `catalog.rs` — local, no server              |
| Tests     | `cargo test` (host), Vitest + happy-dom + Testing Library (UI), Playwright (e2e) |

The chrome page is prerendered, so the sidebar is in the HTML the webview receives — there is no
skeleton because there is no gap for one to fill.

The **site webview** — a second WebView2 child for Microsoft Learn, vendor docs and YouTube, built
the way CleanMyPosts builds it — arrives with M5. That is when `tauri` gains its `unstable` feature
for `Window::add_child`; until then the window hosts one webview.

## Hard rules

1. **Offline first.** Everything works with no network and no account, the catalog included. If a
   server is ever added it unlocks sharing across machines and nothing else — never a precondition
   for studying.
2. **The user's material stays the user's.** Imported files, extracted questions and progress
   live in the local SQLite database. Nothing is uploaded until the user publishes a binder
   explicitly, per binder, with a preview of what leaves the machine.
3. **Extraction is auditable.** Every question carries its source (file, page, byte range) and
   a confidence score. A low-confidence question is flagged for review, never silently
   presented as fact.
4. **No silent quality claims.** If a page was OCR'd, if an image-based question was
   reconstructed from its explanation text, or if a vision model filled a gap, the question is
   labelled with that provenance in the UI.
5. **One credential kind.** The optional assistant/TTS API key lives in the Windows Credential
   Manager, never in a file this app owns, and cannot be read back into the UI.
6. **Nothing is written next to the executable.** Every runtime path comes from Tauri's
   `app_config_dir` / `app_local_data_dir`.
7. **All UI ↔ host communication goes through the bridge**, mirroring the CleanMyPosts
   contract: typed RPC one way, push events the other, never mixed.
8. **No telemetry.**
9. **Interface strings live in `src/lib/i18n/`, never inline in a view.** `en.ts` is the shape and
   `de.ts` must match it key for key — a test walks both and fails on a gap. Question text, options
   and explanations are the user's imported material and stay in the language of their source.

## Commands

```bash
npm run start        # the app itself, in the Tauri window
npm run dev          # UI only, in a browser, against the mock host
npm run build
npm run lint
npm run check
npm run test
npm run app:build    # NSIS installer + updater artifacts
cargo test --manifest-path src-tauri/Cargo.toml
```

## Documentation

Read selectively, not all of it.

| When you work on …                 | read                                                        |
| ---------------------------------- | ----------------------------------------------------------- |
| Product decisions, UX flow         | [00-product-vision.md](.agents/docs/00-product-vision.md)   |
| Projects, layers, WebView2         | [01-architecture.md](.agents/docs/01-architecture.md)       |
| PDF/VCE import, question detection | [03-ingest-pipeline.md](.agents/docs/03-ingest-pipeline.md) |
| Drill sessions, SRS, challenges    | [05-feature-trainer.md](.agents/docs/05-feature-trainer.md) |
| Publish, catalog, ratings, sync    | [16-backend.md](.agents/docs/16-backend.md)                 |
| Order, acceptance criteria         | [14-roadmap.md](.agents/docs/14-roadmap.md)                 |
| Why a decision was made            | [adr/](.agents/docs/adr/)                                   |

# OpenEventViewer

A Windows desktop app that reads the Windows event logs and filters them down to what matters. Two
pages carry the product: Events (a virtualised table over up to fifty thousand records, with a
keyword box, per-column filters, multi-column sort and a bar chart over time) and Diagnose (a
guided scan for the events a machine writes when something went wrong, plus the quarter of an hour
around them).

Desktop is the whole product and there is no server. The app reads the logs Windows already keeps
and writes nothing to them. Nothing is uploaded and there is no account, no API key and no model.

## Language

All documentation, code, comments, commit messages, and diagram labels are written in
**English**.

## Stack

| Layer     | Technology                                                                   |
| --------- | ---------------------------------------------------------------------------- |
| Host      | Tauri 2 (Rust) in `src-tauri/`, one WebView2 window                          |
| Event log | the `windows` crate — `EvtQuery`, `EvtNext`, `EvtRender`, `EvtFormatMessage` |
| Event XML | `roxmltree`, in a pure function with fixtures                                |
| UI        | SvelteKit (prerendered, `adapter-static`), Svelte 5 runes, TypeScript        |
| UI kit    | shadcn-svelte (`new-york`, `neutral`), Tailwind v4, runtime colour presets   |
| Language  | English and German, `src/lib/i18n/`; event text is never translated          |
| Tables    | TanStack Table v8 core, with hand-rolled fixed-height windowing              |
| Contracts | Zod — one source for types and runtime validation                            |
| Updates   | `tauri-plugin-updater`, minisign-signed, no code-signing certificate         |
| Tests     | `cargo test` (host), Vitest + happy-dom + Testing Library (UI)               |

The chrome page is prerendered, so the sidebar is in the HTML the webview receives — there is no
skeleton because there is no gap for one to fill.

## Hard rules

1. **Offline first.** Everything works with no network and no account. Two things reach it, both
   started by the user: the update check at start, and the web search a row's own button opens in
   the default browser. Nothing is sent anywhere on its own.
2. **No credentials.** This app stores none and asks for none. There is no key to leak.
3. **Read-only with respect to the logs.** This app queries the event log and never writes,
   clears or exports a channel.
4. **Nothing is written next to the executable.** Every runtime path comes from Tauri's
   `app_config_dir`; the table's own widths and filters live in `localStorage`.
5. **All UI ↔ host communication goes through the bridge**: `src/lib/bridge/contract.ts` declares
   every command with its arguments and its reply schema, `client.ts` is the only caller, and
   `mock.ts` must keep every command in the contract runnable in a plain browser.
6. **Win32 lives in `eventlog.rs` only**, behind a safe wrapper. Everything that decides
   anything — the filter to XPath, the XML to a record, the incident signatures, the window around
   one — is a pure function with a test.
7. **No telemetry.**
8. **Interface strings live in `src/lib/i18n/`, never inline in a view.** `en.ts` is the shape and
   `de.ts` must match it key for key — a test walks both and fails on a gap. Event messages come
   from the publisher's own resources and stay in the language Windows recorded them in.
9. **Every `Command` goes through `lib.rs::quiet()`**, or the console window it spawns steals the
   webview's input focus and the next click is dropped. A `#[tauri::command]` that blocks runs
   through `commands::blocking()`.

## The Security channel

Reading it needs an elevated process. There is no `requireAdministrator` manifest: the app is
useful without it, and asking every user for elevation to read two channels they mostly do not
want is the wrong trade. `ERROR_ACCESS_DENIED` is mapped to a message that says so, and the Events
page adds a hint about restarting as administrator.

## Releases

The updater verifies a minisign signature, not an Authenticode one. The key pair is generated once
by hand:

```bash
npx tauri signer generate -w ~/.tauri/openeventviewer.key
```

The public half goes into `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`; the private half
and its password are the GitHub secrets `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` and never enter the repository. `release.yml` builds on a `v*`
tag and uploads the installer, its `.sig` and a `latest.json` written from that signature.

## Commands

```bash
npm run start        # the app itself, in the Tauri window
npm run dev          # UI only, in a browser, against the mock host
npm run build
npm run lint
npm run check
npm run test
npm run licenses     # regenerates src/lib/third-party.json and the shipped notices
npm run app:build    # NSIS installer + updater artifacts
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml -- --ignored   # reads the real event log
```

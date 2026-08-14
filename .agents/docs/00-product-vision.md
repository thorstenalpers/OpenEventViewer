# Product vision

## The loop

Import material → drill it → everything wrong becomes the next session → the binder grows with
the links, docs, videos and notes that fixed each gap → export it, share it, or challenge someone
to beat your time on it.

## The binder ("Lernmappe")

The binder is the unit of everything: the thing you study, export, publish, rate and import. It is
one folder and one manifest, not a scattering of app state.

```
AZ-900.examdeck                 (zip)
  manifest.json                title, certification, version, author, license, tags, counts
  questions.json               questions, options, answers, explanations, provenance
  links.json                   documentation links, learning-portal bookmarks
  media.json                   video entries (YouTube ids / URLs) with per-question anchors
  notes/                       markdown notes, per question or free-standing
  assets/                      figures, screenshots, generated podcast audio
  sources/                     optional: the original PDF, if the author chooses to include it
```

Exporting is writing that zip. Importing is reading it. Publishing is uploading it. The catalog
shows manifests; the download is the same zip. One format, four features.

## The surfaces

| Surface    | What it is                                                                                                 |
| ---------- | ---------------------------------------------------------------------------------------------------------- |
| Library    | your binders — TanStack Table, sortable and filterable by certification, progress, due count, last studied |
| Import     | drop a PDF or VCE, watch extraction, land in Review                                                        |
| Review     | fix what the extractor was unsure about, side by side with the source page                                 |
| Train      | the drill session                                                                                          |
| Browse     | the site webview — Microsoft Learn, vendor docs, learning portals, logged in as you                        |
| Media      | videos attached to the binder or to single questions; the podcast generator                                |
| Assistant  | AI on the current question, the current binder, or the extraction                                          |
| Catalog    | public binders — browse, filter, rate, import, take on challenges                                          |
| Statistics | per certification, per topic, per question: accuracy, speed, streaks                                       |
| Settings   | theme, assistant source, TTS voice, account, storage, timings                                              |

## Browsing is part of studying, not a link-out

The site webview is a real logged-in browser, the same construction as CleanMyPosts: a second
WebView2 child inside the same window, with its own persistent profile. You sign in to Microsoft
Learn once and stay signed in. Every `Reference:` URL the extractor found is a bookmark in the
binder, so "read the doc behind this question" is one click and never leaves the app.

## Videos

A binder carries video entries — YouTube ids or plain URLs — either at binder level ("the course")
or anchored to a single question ("the 4 minutes that explain this"). Anchors carry a start
timestamp. Playback happens in the site webview.

## Podcast

Select questions — a topic, a wrong-answer set, a whole binder — and generate an audio episode:
question, a configurable pause, the correct answer, then the explanation, with the option to omit
answers entirely for a pure recall drill. Output is an audio file plus chapter marks, playable in
the app and exportable to a phone. Offline voice by default (Windows speech synthesis, no key,
nothing leaves the machine). A better voice is a download away rather than an account: neural voice
packs are fetched on request into the app's own data folder and read every episode from there — the
only thing that leaves the machine is the request for the pack itself.

## Assistant

Same shape as CleanMyPosts: the local `claude` binary is the default source and sends nothing to a
third party; hosted providers are opt-in and their key lives in the Windows Credential Manager.
What it is asked to do here:

- explain why the marked answer is right and the tempting one is wrong
- generate variants of a question you keep failing, so you learn the concept, not the wording
- turn an explanation into a note, and a set of notes into a summary
- reconstruct an image-based question the extractor could not (labelled as model-generated)
- suggest documentation links for a topic with a weak score

## Accounts

Optional, always. No account: everything above works, progress is local, export is a file you
hand to someone. With an account: progress syncs across machines, binders can be published to the
catalog, ratings and challenge results are attributable.

## Challenges

A challenge is a binder plus a rule set: question order seed, time limit, whether explanations are
shown. Results record accuracy and elapsed time per question, so a leaderboard compares like with
like. Local challenges need no account; published ones do.

## What this app is not

Not a dump distributor. The catalog carries binders that users publish, and publishing braindump
content that a certification vendor considers confidential is the publisher's call and the
publisher's exposure. The app's own defaults are local and private; sharing is a deliberate,
per-binder action with a preview of exactly what would be uploaded.

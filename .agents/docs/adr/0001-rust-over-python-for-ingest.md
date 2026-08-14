# ADR 0001 — Rust, not Python, for the ingest pipeline

Status: accepted

## Context

The import pipeline has to read PDF dumps and VCE files, recover question structure from
layout, extract embedded images, and render page regions as images for questions whose content
is graphical. Python has the better-known document tooling, so the choice needs stating.

## Decision

The ingest pipeline is Rust, inside the Tauri host. No Python runtime is shipped.

## Reasons

- **Shipping.** The product is a desktop app. A Python pipeline means a PyInstaller sidecar of
  50–80 MB, a second thing to code-sign, and a recurring source of antivirus false positives on
  Windows. Rust compiles into the binary that already exists.
- **PDFium covers the whole job.** `pdfium-render` gives per-character text with bounding boxes,
  embedded image XObjects, and page-region rasterisation from one library. The layout-aware
  extraction that Section 2 of [03-ingest-pipeline.md](../03-ingest-pipeline.md) depends on needs
  coordinates, and it has them.
- **Licensing.** The Python option that actually wins on quality is PyMuPDF, which is AGPL. This
  project is MIT. `pdfium-render` wraps PDFium (BSD-3).
- **The hard part is not the library.** Question recovery is a state machine over positioned text
  runs plus per-provider heuristics. That code is roughly the same length in either language, and
  it is easier to test and keep fast in Rust.

## Where Python would have won, and why it does not apply

Deep-learning layout analysis (LayoutParser, docTR, table transformers) is Python-only in
practice. These dumps do not need it: they are single-column, machine-generated PDFs with a rigid
`NEW QUESTION n` / options / `Answer:` / `Explanation:` skeleton. If a future source needs real
layout ML, the escape hatch is a vision model call through the assistant, not a bundled runtime.

## Consequences

- OCR, when needed, comes from `ocrs` (pure Rust) or an optional Tesseract the user already has —
  not from a bundled Python stack.
- Image-only questions are reconstructed by an optional vision-model pass, opt-in and labelled.

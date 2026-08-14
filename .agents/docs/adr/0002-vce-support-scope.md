# ADR 0002 — What "VCE support" means

Status: accepted

## Context

Two of the four sample files are `.vce` — the Avanset VCE Exam Simulator format. Both begin
with the same 8-byte signature and are opaque:

```
85 a8 06 02 04 00 00 00   Microsoft.examanswers.AZ-900.v2025-05-27.by.harvey.179q.vce
85 a8 06 02 04 00 00 00   Microsoft.PracticeTest.AZ-900.v2021-12-31.by.Whitley.174q.vce
```

Measured Shannon entropy of the first file is **7.998 bits/byte** over 1.36 MB, and the file
contains no printable run that is not random noise. That is encrypted or encrypted-then-compressed
data, not an obfuscated container. There is no published specification.

Note that the other two files are named `*.vce.pdf` — they are ordinary PDFs, and only the naming
suggests otherwise. They go through the PDF path.

## Decision

VCE is a **best-effort importer behind a capability probe**, never an advertised guarantee.

The importer reads the header, identifies the container version, and dispatches to a decoder
if one exists for that version. When no decoder matches, the import fails with a specific,
honest message and offers the documented alternative: export the exam to PDF from a VCE player
the user already owns, then import that PDF.

## Rules

1. **Never claim VCE support in UI, README or release notes without naming the versions that
   actually decode.** "VCE (selected versions)" with a link to the support matrix.
2. **No decoder is merged without a fixture** — a real file that round-trips to questions in
   `cargo test`.
3. Decoder work is interoperability with the user's own purchased material on their own machine.
   It stays that: no license-server work, no player emulation, nothing that redistributes content.
4. If the probe cannot identify a version, the file is left untouched and the failure names the
   signature bytes so the support matrix can grow from real reports.

## Consequences

- The roadmap treats PDF as the load-bearing path and VCE as an additive one. Nothing in the
  trainer, catalog or export depends on VCE working.
- The support matrix lives in `.agents/docs/03-ingest-pipeline.md` and is updated per decoder.

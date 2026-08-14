# Ingest pipeline

Turning an exam file into reviewable questions. This is the part of the product whose quality
decides whether the rest is worth using, so it is specified in more detail than anything else.

Location: `crates/ingest/` (`openexamtrainer-ingest`).

```
src/
  lib.rs          orchestration: detect profile → strip furniture → order → parse → score
  model.rs        TextLine, Document, Question, Warning, ExtractionReport
  pdf.rs          pdfium glyphs with coordinates          [feature = "pdfium"]
  figures.rs      rasterises the figure band, masks, crops, hashes  [feature = "pdfium"]
  layout.rs       line assembly, header/footer removal, column detection, reading order
  parse.rs        the question state machine
  bank.rs         the second source: Markdown/JSON question banks (see §10)
  profiles.rs     per-vendor detectors and overrides
  confidence.rs   scoring, warnings, missing-number detection
  vce.rs          version probe + decoder dispatch (see adr/0002)
  bin/examingest.rs  CLI: run the pipeline over one file, `--json` for the full report
```

Everything except `pdf.rs` and `figures.rs` is free of the PDF backend and tested with synthetic
`TextLine`s, so the state machine, the furniture model and the bank parser can be developed and
regression-tested without pdfium present. `cargo test --no-default-features` is that path.

## 1. Source facts measured on the sample files

These numbers come from the four files in `~/Desktop/Exams` and drive the design.

| File                              | Profile      | Pages | Real questions | Notes                                                                                 |
| --------------------------------- | ------------ | ----- | -------------- | ------------------------------------------------------------------------------------- |
| `pass4sure.ai-900…112q.vce.pdf`   | `certshared` | 6     | **8**          | 5 image-type, all 5 figures recovered; marker 9 absent, marker 10 is the paywall stub |
| `transcender.ai-900…250q.vce.pdf` | `certleader` | 7     | **11**         | 5 image-type, all 5 recovered; two markers numbered 10, then a jump to the stub at 15 |
| `examanswers.AZ-900…179q.vce`     | —            | —     | —              | encrypted, entropy 7.998                                                              |
| `PracticeTest.AZ-900…174q.vce`    | —            | —     | —              | encrypted, same signature                                                             |

**The `112q` and `250q` in the filenames are marketing, not content.** Both PDFs are free teasers.
Each ends with a `NEW QUESTION n` marker whose body is an ellipsis and an advertisement — a
truncation notice, not a question. Those are reported as `stub_markers`, so "this file is an
excerpt" is a fact the UI can state rather than a shortfall that looks like an extraction bug.

Neither source numbers its questions reliably: one skips a number, the other repeats one. The
parser follows the source and records the anomaly; it never renumbers to make the output look tidy.

## 2. Text extraction is coordinate-based, not `pdftotext`

Naive text extraction interleaves page furniture into question content. Real output from the
pass4sure file:

```
A. Mastered                              visit - https://www.certshared.com
B. Not Mastered
```

The footer landed inside option A. Any regex working on flat text inherits that corruption.

So: PDFium yields every character with `(x, y, w, h)` and its scaled font size, and `layout.rs`
assembles the lines itself. Pdfium's own segment boundaries are not used — a segment does not know
that the run to its right is a footer.

Pdfium initialises process-global state and rejects a second initialisation, so `pdf::read_file`
owns a singleton and serialises reads through it. Binding it twice in one process is an error, not
a slowdown; two tests doing it concurrently abort the harness.

1. **Furniture detection.** Cluster runs by normalised Y across all pages. A cluster occupying the
   same band on ≥ 60 % of pages with near-identical text is furniture — header, footer, watermark,
   page number. Drop it before anything else looks at the text.
2. **Column detection.** X-histogram over run starts; a clear bimodal gap means two columns. These
   dumps are single-column, but vendor study guides are not.
3. **Reading order.** Sort by column, then Y descending, then X — never by the PDF's content-stream
   order, which is arbitrary.
4. **Line assembly.** Merge runs whose baselines agree within a tolerance derived from font size,
   not a fixed pixel constant.

## 3. The question state machine

Operates on assembled lines, in reading order, as an explicit state machine — not a monolithic
regex.

```
Idle ──"NEW QUESTION 12"──▶ Head ──"- (Exam Topic 1)"──▶ Stem
Stem ──"A. …"──▶ Options ──"Answer: BD"──▶ Answer ──"Explanation:"──▶ Explanation
Explanation ──"Reference:"──▶ References ──next question marker──▶ Head
```

Anchors, all case-insensitive and all profile-overridable:

| Element   | Pattern                                                           |
| --------- | ----------------------------------------------------------------- |
| Marker    | `^(NEW\s+)?QUESTION\s*#?\s*(\d+)` \| `^Question\s+\d+\s+of\s+\d+` |
| Topic     | `^-?\s*\(Exam Topic (\d+)\)`                                      |
| Option    | `^([A-Z])[.)]\s+` — must be sequential from `A`                   |
| Answer    | `^Answer:\s*([A-Z][A-Z,\s]*)`                                     |
| Rationale | `^Explanation:` \| `^Explanation/Reference:`                      |
| Reference | `^Reference:` or a bare URL inside the explanation                |

**Question type is derived, never guessed:**

- one answer letter, ≥ 2 options → `SingleChoice`
- ≥ 2 answer letters → `MultipleChoice`, and the letter count is the required selection count
- options are exactly `A. Mastered` / `B. Not Mastered` → `ImageBased` (see §4)
- explanation contains `Box n:` lines → `Matrix`, parsed into ordered box/value pairs

Sequence integrity: marker numbers must ascend by one. A gap means a question was missed —
raise it as a warning with the page number rather than silently producing fewer questions.

## 4. Image-based questions

`A. Mastered / B. Not Mastered` is the dump format's stand-in for a drag-and-drop, hotspot or
yes/no-matrix question whose real content is a picture. Five of the eight recovered questions in
`certshared-ai900.pdf`, and five of eleven in `certleader-ai900.pdf`, are of this type.

> **Corrected 2026-08-12.** An earlier revision of this document claimed these files contain "no
> embedded images at all" and that "the screenshots were stripped from the free sample". That was
> wrong. It came from `grep -c "/Image"` over the raw file, which cannot see into compressed object
> streams. Rendering the bands proves the figures are there: all ten are recovered, and each is a
> complete answer area. Never conclude a PDF lacks something from a grep over its bytes.

The figures carry no text layer of their own, so the extractor finds them geometrically:

1. **A visual gap** — the band between the last stem line and the first option. Rasterise it at
   200 dpi, crop, hash, attach. This recovers raster screenshots and vector-drawn tables alike,
   which is why the band is rendered rather than lifted out as an XObject.
2. **The gap spans a page break** — the figure is what pushed the options overleaf, so the band is
   the head of the _option's_ page, not the tail of the stem's.
3. **Nothing there** — the question is _incomplete_. Reconstruct what can be reconstructed from the
   explanation (the `Box 1: No / Box 2: Yes` lines are the answer key) and mark the question
   `needs_source`. It stays in the binder, is shown in review, and is excluded from scored
   sessions until the user supplies the missing figure.

A question that gets its figure back is no longer `needs_source`: it has both a picture and an
answer key, so it is scored like any other. Rule 4 in [AGENTS.md](../../AGENTS.md) applies to the
rest — an incomplete question is labelled as such in the UI, never dressed up as a normal one.

Two mask sets keep the capture honest, and they are deliberately not one list:

- **Furniture strips** — the header and footer, as full-width geometry rather than as lines. A
  vendor's running header is a logo and a rule as much as a sentence, and neither carries a
  `TextLine`. The strip's depth comes from where the header's own lines end, not from the 12 %
  candidate zone of §2 — that zone only _nominates_ repeated lines and is far too deep to cut with.
  Furniture is excluded from both the measurement and the crop.
- **Text lines** — already extracted and displayed above the figure. They must not count toward
  "is there a diagram here", or a footer between stem and options reads as one. They are still
  cropped _in_: an `Answer Area` heading and the sentence a dropdown sits inside are what make the
  picture legible.

Assets are content-addressed by SHA-256 and stored once per hash under the app data directory, so
two questions quoting the same answer area share a file, and re-importing a dump rewrites the same
bytes instead of accumulating copies. `.examdeck` carries the PNGs under `figures/`; without them
a deck's hashes resolve to nothing on the machine that opens it.

## 5. Provider profiles

Each dump vendor renders the same skeleton differently. A profile is a detector plus overrides.

```rust
struct Profile {
    id: &'static str,
    detect: fn(&[Page]) -> bool,   // e.g. footer contains "certshared.com"
    marker: Regex,
    furniture: Vec<Regex>,
    strip_trailing_url: bool,
}
```

Shipping profiles: `certshared`, `certleader`, `generic`. `generic` runs when nothing matches and
relies purely on the anchors in §3. Adding a vendor is a profile, never a change to `parse.rs`.

## 6. Confidence and review

Every question gets a score in `0.0..=1.0`. Deductions:

| Signal                                             | Deduction |
| -------------------------------------------------- | --------- |
| Marker number out of sequence                      | 0.30      |
| Option letters not sequential from `A`             | 0.25      |
| Answer letter has no matching option               | 0.40      |
| No answer found                                    | 0.50      |
| Stem shorter than 20 characters                    | 0.30      |
| `needs_source` (image-based, figure not recovered) | 0.35      |
| Text recovered by OCR rather than the text layer   | 0.20      |

Scoring runs once after parsing and again after figure capture, because capture clears
`needs_source` on every question it recovers. Re-running the whole scorer rather than subtracting
the penalty by hand keeps one definition of confidence.

Below 0.75 the question lands in the **Review** view: source page rendered on the left, parsed
fields editable on the right, keyboard-driven accept/fix. Corrections are stored and, where the
correction is a repeatable pattern, offered as a profile rule.

## 7. Provenance

Every question stores `source_file`, `source_page`, `source_span`, `profile_id`, `extractor_version`,
`confidence`, and `provenance` (`text_layer` | `ocr` | `vision_model` | `manual`). Re-importing a
newer version of the same file diffs against the stored provenance instead of duplicating the binder.

## 8. VCE support matrix

See [adr/0002](adr/0002-vce-support-scope.md) for scope. Current state:

| Signature (first 8 bytes) | Container       | Decoder                                                       |
| ------------------------- | --------------- | ------------------------------------------------------------- |
| `85 a8 06 02 04 00 00 00` | Avanset, opaque | none — probe reports unsupported, import offers the PDF route |

No row is added to this table without a passing fixture test.

## 9. Fixtures and how to run the pipeline

The regression suite in `crates/ingest/tests/real_dumps.rs` runs against real vendor PDFs. Those
are the user's own material and are **not** committed — `crates/ingest/tests/fixtures/` is
gitignored, and a missing fixture makes the test print `SKIPPED` rather than pass silently on
nothing. Expected filenames:

```
crates/ingest/tests/fixtures/certshared-ai900.pdf
crates/ingest/tests/fixtures/certleader-ai900.pdf
crates/ingest/tests/fixtures/avanset-az900.vce
```

Pdfium is likewise not committed. Fetch it once:

```bash
pwsh scripts/fetch-pdfium.ps1
```

Then:

```bash
cargo test --no-default-features                        # parser and layout, no native library
cargo test                                              # plus the real-PDF regressions
cargo run --bin examingest -- <file.pdf>                 # one file, human-readable
cargo run --bin examingest -- <file.pdf> --json          # the full report
```

The strongest assertion in the suite is the negative one: no question's stem, option or explanation
may contain the vendor's own name. That is the corruption of §2, and it is checked on every run.

## 10. Question banks (Markdown and JSON)

The second source, and the one that does not require a dump. Openly published banks — the ones
whose authors state the questions were _not_ transcribed from a real exam — are all plain text, so
`bank.rs` needs no PDF backend and is tested without one.

Everything downstream is shared: the same `Question`, the same confidence scorer, the same review
flow, the same trainer. Only field recovery differs.

### The shape the banks actually use

Measured on two published AI-900 banks, not invented:

```html
<h5>56. What are three Microsoft guiding principles for responsible AI?</h5>
<ol type="a">
	<li>inclusiveness</li>
	...
</ol>
<details>
	<summary>Show Answer</summary>
	<p>['inclusiveness', 'fairness', 'reliability and safety']</p>
</details>
```

Three things that only reading the real files reveals:

1. **The multi-answer key is a Python list literal.** Nobody designed that; it fell out of the
   author's script. Not reading it turns a three-of-six question into one nonsense answer whose
   text is `['inclusiveness', …]`.
2. **Quote style varies between banks** — `type='a'` and `type="a"` — so nothing may key off it.
3. **The key is a text, not a letter,** and is occasionally annotated: one bank answers `AI (.ai)`
   where its option reads `AI`.

Plain-Markdown equivalents (`##### `, `1. `, `**Answer:**`) are accepted for the same roles.

### Resolving the key

In order: exact text match after normalisation → a bare single letter → a prefix match that hits
**exactly one** option. The prefix pass is what handles `AI (.ai)`; its uniqueness guard is what
keeps it a determination rather than a guess. `Azure Machine Learning` against a bank that also
offers `Azure Machine Learning Studio` matches twice and is therefore left unanswered.

An unresolved key is never invented. The question is stored with no correct option, the scorer
flags it `MissingAnswer`, and it lands in Review for the user to settle — a bank with a typo in its
key is something to look at, not something to paper over.

### JSON

For hand-authored banks and for other tools:

```json
[
	{
		"question": "Which service trains models?",
		"options": ["Azure Machine Learning", "Azure Bot Service"],
		"answer": "Azure Machine Learning",
		"explanation": "optional",
		"references": ["optional"]
	}
]
```

`answer` takes a string or a list; `stem`/`text`, `choices`, `answers`/`correct` and `links` are
accepted as aliases. Text is matched before letters, so an option whose own text is `B` stays
reachable.

### Measured result

| Bank                                                                                         | Questions | Multi-answer | Key unresolved | In review |
| -------------------------------------------------------------------------------------------- | --------- | ------------ | -------------- | --------- |
| [harshpandita2000](https://github.com/harshpandita2000/Azure-AI-900-Practice-Questions-2024) | 139       | 2            | 0              | 0         |
| [olafwrieden](https://github.com/olafwrieden/Azure-AI-900-Practice-Questions)                | 29        | 0            | 0              | 0         |

Re-measure with `crates/ingest/tests/bank_real.rs`, which reads whatever `OET_BANK_DIR` points at.
The banks are not vendored — they are other people's repositories, with their own licences.

### What this path deliberately does not do

It does not fetch. A bank arrives as a file the user chose, like every other import. Nothing in
this app reaches the network to acquire study material.

#![cfg(feature = "pdfium")]

//! Writes every captured figure to `target/figures/` so a human can look at what the extractor
//! decided was a diagram. Not an assertion — a way to check the heuristic against reality.
//!
//! Run with: `cargo test -p openexamtrainer-ingest --test dump_figures -- --ignored --nocapture`

use std::path::Path;

use openexamtrainer_ingest::{extract, figures, pdf};

#[test]
#[ignore = "diagnostic, not a check"]
fn dump() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/figures");
    std::fs::create_dir_all(&out).expect("out dir");

    for name in ["certshared-ai900.pdf", "certleader-ai900.pdf"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        if !path.exists() {
            eprintln!("SKIPPED {name}");
            continue;
        }

        let document = pdf::read_file(&path, &[pdf::vendored_library_dir()]).expect("read");
        let mut report = extract(&document);
        let flagged: Vec<(u32, Option<_>)> = report
            .questions
            .iter()
            .filter(|q| q.needs_source)
            .map(|q| (q.number, q.figure_band))
            .collect();
        println!("\n== {name}: {} flagged", flagged.len());
        for (number, band) in &flagged {
            println!("  q{number:02} band {band:?}");
        }
        let assets = figures::capture(&path, &document, &mut report).expect("capture");

        for question in &report.questions {
            for hash in &question.figures {
                let asset = assets.iter().find(|a| &a.hash == hash).expect("asset");
                let file = out.join(format!(
                    "{}-q{:02}-{}.png",
                    name.split('-').next().unwrap_or(name),
                    question.number,
                    &hash[..8]
                ));
                std::fs::write(&file, &asset.png).expect("write");
                println!("{} ({} bytes)", file.display(), asset.png.len());
            }
        }
    }
}

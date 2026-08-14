//! Runs the bank parser over real published files and reports what it recovered.
//!
//! The banks are other people's repositories with their own licences, so they are not vendored.
//! Point `OET_BANK_DIR` at a directory of downloaded `.md` / `.json` files instead.
//!
//! ```text
//! OET_BANK_DIR=… cargo test -p openexamtrainer-ingest --no-default-features \
//!     --test bank_real -- --ignored --nocapture
//! ```

use openexamtrainer_ingest::bank::{self, Format};
use openexamtrainer_ingest::confidence::REVIEW_THRESHOLD;

#[test]
#[ignore = "diagnostic, needs OET_BANK_DIR"]
fn measure() {
    let Some(dir) = std::env::var_os("OET_BANK_DIR") else {
        eprintln!("SKIPPED: set OET_BANK_DIR to a directory of question banks");
        return;
    };

    for entry in std::fs::read_dir(dir).expect("dir") {
        let path = entry.expect("entry").path();
        let Some(format) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(Format::of)
        else {
            continue;
        };

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let text = std::fs::read_to_string(&path).expect("read");
        let report = match bank::parse(&text, format) {
            Ok(report) => report,
            Err(error) => {
                println!("\n== {name}: REFUSED — {error}");
                continue;
            }
        };

        let unresolved: Vec<u32> = report
            .questions
            .iter()
            .filter(|question| question.answer_letters.is_empty())
            .map(|question| question.number)
            .collect();

        println!("\n== {name}");
        println!("  questions:  {}", report.questions.len());
        println!(
            "  multi:      {}",
            report
                .questions
                .iter()
                .filter(|q| q.answer_letters.len() > 1)
                .count()
        );
        println!(
            "  review:     {}",
            report
                .questions
                .iter()
                .filter(|q| q.confidence < REVIEW_THRESHOLD)
                .count()
        );
        println!("  no answer:  {} {unresolved:?}", unresolved.len());

        // An unresolved key is the one thing worth looking at by hand: it is either a typo in the
        // bank or a shape the parser does not yet read, and only the text tells you which.
        for number in unresolved.iter().take(3) {
            let question = report
                .questions
                .iter()
                .find(|q| q.number == *number)
                .expect("listed above");
            println!("  --- #{number}: {}", question.stem);
            for option in &question.options {
                println!("        {}. {}", option.letter, option.text);
            }
        }
    }
}

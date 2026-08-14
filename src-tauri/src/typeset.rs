//! A Markdown summary, set on A4 pages.
//!
//! `pdfium` reads PDFs and cannot write one, which is why M13 shipped without this. The library
//! that would fill the gap brings seventy-odd crates for one button, so the file is written here
//! instead: a text-only PDF over a built-in font is a page of structure, and every claim this
//! module makes about the result is checked by opening it again with the reader the app already
//! carries.
//!
//! One typeface at three sizes rather than a roman and a bold. Wrapping needs the advance of every
//! glyph it sets, and a second set of metrics is a second table to get wrong — a heading reads as a
//! heading from its size alone.

/// Adobe's advances for Helvetica, in thousandths of an em, for the printable ASCII range.
///
/// The PDF names the font rather than embedding it, so these are the numbers the viewer will use.
/// Measuring with anything else would wrap lines to a width the page does not have.
const ASCII_WIDTHS: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // 32..47
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, // 48..63
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, // 64..79
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, // 80..95
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, // 96..111
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584, // 112..126
];

/// What a glyph outside the ASCII table costs. The accented letters this app meets are Latin-1
/// vowels, all of which are 556 in Helvetica; being a few per cent out on a rarer one costs a
/// slightly short line, not an overflowing one.
const DEFAULT_WIDTH: u16 = 556;

const PAGE_WIDTH: f32 = 595.28;
const PAGE_HEIGHT: f32 = 841.89;
const MARGIN: f32 = 56.7;
const BODY_SIZE: f32 = 10.5;
const LEADING: f32 = 1.45;

/// A line, once it knows how big it is and how far in it starts.
struct Line {
    text: String,
    size: f32,
    indent: f32,
    /// Blank space above, for the gap that separates a heading from what came before it.
    space_above: f32,
}

/// Turns one Markdown line into its size and indent. Headings by `#`, list items by their marker,
/// everything else body text.
fn style(raw: &str) -> (String, f32, f32, f32) {
    let trimmed = raw.trim_end();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if (1..=3).contains(&hashes) && trimmed.chars().nth(hashes) == Some(' ') {
        let size = match hashes {
            1 => BODY_SIZE + 6.5,
            2 => BODY_SIZE + 3.5,
            _ => BODY_SIZE + 1.5,
        };
        return (
            trimmed[hashes + 1..].trim().to_string(),
            size,
            0.0,
            size * 0.9,
        );
    }

    let body = trimmed.trim_start();
    let bullet = body
        .strip_prefix("- ")
        .or_else(|| body.strip_prefix("* "))
        .or_else(|| body.strip_prefix("+ "));
    if let Some(rest) = bullet {
        return (format!("• {}", rest.trim()), BODY_SIZE, 14.0, 0.0);
    }
    // A numbered item keeps its own number: renumbering someone's list would change what it says.
    let numbered = body
        .split_once(". ")
        .filter(|(head, _)| !head.is_empty() && head.chars().all(|c| c.is_ascii_digit()));
    if let Some((number, rest)) = numbered {
        return (format!("{number}. {}", rest.trim()), BODY_SIZE, 14.0, 0.0);
    }

    (body.to_string(), BODY_SIZE, 0.0, 0.0)
}

/// Strips the emphasis markers rather than honouring them: this sets one typeface, so a `**` left
/// standing would be read as two asterisks the author never typed.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' | '_' => {
                if chars.peek() == Some(&c) {
                    chars.next();
                }
            }
            '`' => {}
            _ => out.push(c),
        }
    }
    out
}

fn width_of(text: &str, size: f32) -> f32 {
    let thousandths: u32 = text
        .chars()
        .map(|c| {
            let index = c as u32;
            if (32..127).contains(&index) {
                u32::from(ASCII_WIDTHS[(index - 32) as usize])
            } else {
                u32::from(DEFAULT_WIDTH)
            }
        })
        .sum();
    thousandths as f32 * size / 1000.0
}

/// Breaks a paragraph at spaces, and inside a word when a single one is wider than the column —
/// a URL with no spaces in it would otherwise run off the page rather than wrap.
fn wrap(text: &str, size: f32, budget: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if width_of(&candidate, size) <= budget {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if width_of(word, size) <= budget {
            current = word.to_string();
            continue;
        }
        let mut piece = String::new();
        for c in word.chars() {
            if width_of(&format!("{piece}{c}"), size) > budget && !piece.is_empty() {
                lines.push(std::mem::take(&mut piece));
            }
            piece.push(c);
        }
        current = piece;
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn lay_out(markdown: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    for raw in markdown.lines() {
        if raw.trim().is_empty() {
            lines.push(Line {
                text: String::new(),
                size: BODY_SIZE,
                indent: 0.0,
                space_above: 0.0,
            });
            continue;
        }
        // A rule is what `workshop::collect` puts between two notes; a bar of it would be furniture.
        if raw.trim().chars().all(|c| c == '-' || c == '*') && raw.trim().len() >= 3 {
            continue;
        }

        let (body, size, indent, space_above) = style(raw);
        let budget = PAGE_WIDTH - 2.0 * MARGIN - indent;
        for (index, piece) in wrap(&plain(&body), size, budget).into_iter().enumerate() {
            lines.push(Line {
                text: piece,
                size,
                indent,
                space_above: if index == 0 { space_above } else { 0.0 },
            });
        }
    }
    lines
}

/// WinAnsi is what the font resource declares, so a character it has no byte for cannot be written.
/// The typographic ones the assistant produces get their WinAnsi byte; anything else becomes `?`,
/// visibly, rather than silently disappearing out of somebody's notes.
fn encode(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() + 8);
    for c in text.chars() {
        let byte = match c {
            '(' | ')' | '\\' => {
                out.push(b'\\');
                c as u8
            }
            '\u{2018}' => 0x91,
            '\u{2019}' => 0x92,
            '\u{201C}' => 0x93,
            '\u{201D}' => 0x94,
            '\u{2022}' => 0x95,
            '\u{2013}' => 0x96,
            '\u{2014}' => 0x97,
            '\u{2026}' => 0x85,
            '\u{20AC}' => 0x80,
            c if (0x20..0x7F).contains(&(c as u32)) => c as u8,
            c if (0xA0..0x100).contains(&(c as u32)) => c as u8,
            _ => b'?',
        };
        out.push(byte);
    }
    out
}

/// Sets a Markdown summary as an A4 PDF and returns the file's bytes.
pub fn markdown_to_pdf(title: &str, markdown: &str) -> Vec<u8> {
    let mut pages: Vec<Vec<u8>> = Vec::new();
    let mut stream: Vec<u8> = Vec::new();
    let mut cursor = PAGE_HEIGHT - MARGIN;
    let floor = MARGIN;

    for line in lay_out(markdown) {
        let step = line.size * LEADING;
        cursor -= line.space_above;
        if cursor - step < floor {
            pages.push(std::mem::take(&mut stream));
            cursor = PAGE_HEIGHT - MARGIN;
        }
        cursor -= step;
        if line.text.is_empty() {
            continue;
        }

        stream.extend_from_slice(b"BT\n/F1 ");
        stream.extend_from_slice(format!("{:.2} Tf\n", line.size).as_bytes());
        stream.extend_from_slice(
            format!("{:.2} {:.2} Td\n(", MARGIN + line.indent, cursor).as_bytes(),
        );
        stream.extend_from_slice(&encode(&line.text));
        stream.extend_from_slice(b") Tj\nET\n");
    }
    pages.push(stream);

    assemble(title, &pages)
}

/// Writes the object graph and the cross-reference table around the page streams.
///
/// The offsets in that table are counted while the file is built rather than worked out afterwards:
/// a reader finds every object through it, and one wrong number is a file that will not open.
fn assemble(title: &str, pages: &[Vec<u8>]) -> Vec<u8> {
    let font_id = 3;
    let first_page_id = 4;
    let object_count = 3 + pages.len() * 2;

    let mut out: Vec<u8> = Vec::new();
    let mut offsets: Vec<usize> = vec![0; object_count + 1];
    out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

    let object = |out: &mut Vec<u8>, offsets: &mut Vec<usize>, id: usize, body: &[u8]| {
        offsets[id] = out.len();
        out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };

    object(
        &mut out,
        &mut offsets,
        1,
        b"<< /Type /Catalog /Pages 2 0 R >>",
    );

    let kids: String = (0..pages.len())
        .map(|index| format!("{} 0 R ", first_page_id + index * 2))
        .collect();
    object(
        &mut out,
        &mut offsets,
        2,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.trim_end(),
            pages.len()
        )
        .as_bytes(),
    );

    object(
        &mut out,
        &mut offsets,
        font_id,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
    );

    for (index, stream) in pages.iter().enumerate() {
        let page_id = first_page_id + index * 2;
        let content_id = page_id + 1;
        object(
            &mut out,
            &mut offsets,
            page_id,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.2} {PAGE_HEIGHT:.2}] \
                 /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
            )
            .as_bytes(),
        );

        let mut body = format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes();
        body.extend_from_slice(stream);
        body.extend_from_slice(b"endstream");
        object(&mut out, &mut offsets, content_id, &body);
    }

    let start_xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", object_count + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info << /Title (",
            object_count + 1
        )
        .as_bytes(),
    );
    out.extend_from_slice(&encode(title));
    out.extend_from_slice(b") /Producer (OpenExamTrainer) >> >>\n");
    out.extend_from_slice(format!("startxref\n{start_xref}\n%%EOF\n").as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUMMARY: &str = "# AZ-900 Zusammenfassung\n\nDie Notizen zu diesem Kurs — kurz \
        gefasst, mit Umlauten: Größe, Verfügbarkeit, Ausfälle.\n\n## Speicher\n\n- Blob Storage \
        hält unstrukturierte Daten\n- Ein Konto trägt mehrere Dienste\n\n## Links\n\n\
        https://learn.microsoft.com/en-us/credentials/certifications/resources/study-guides/az-900\n";

    #[test]
    fn a_heading_is_told_from_a_list_item_and_from_a_paragraph() {
        assert_eq!(style("## Speicher").1, BODY_SIZE + 3.5);
        assert_eq!(style("- first").0, "• first");
        assert_eq!(style("3. third").0, "3. third");
        assert_eq!(style("3.14 is pi").0, "3.14 is pi");
        // A bare hash is not a heading, and neither is a fourth level this sets no size for.
        assert_eq!(style("#hashtag").1, BODY_SIZE);
        assert_eq!(style("#### deeper").1, BODY_SIZE);
    }

    #[test]
    fn emphasis_markers_are_removed_rather_than_printed() {
        assert_eq!(
            plain("**bold** and _quiet_ and `code`"),
            "bold and quiet and code"
        );
    }

    #[test]
    fn no_wrapped_line_is_wider_than_its_column() {
        let budget = PAGE_WIDTH - 2.0 * MARGIN;
        let long = "Azure Blob Storage is the object store for unstructured data, and it is the \
                    one every other service ends up writing into sooner or later.";

        for line in wrap(long, BODY_SIZE, budget) {
            assert!(width_of(&line, BODY_SIZE) <= budget, "{line}");
        }
    }

    /// A URL has nowhere to break, so it has to be cut rather than left hanging off the page.
    #[test]
    fn a_word_wider_than_the_column_is_broken_inside() {
        let budget = 60.0;
        let pieces = wrap(
            "https://learn.microsoft.com/en-us/credentials",
            BODY_SIZE,
            budget,
        );

        assert!(pieces.len() > 1);
        for piece in &pieces {
            assert!(width_of(piece, BODY_SIZE) <= budget, "{piece}");
        }
        assert_eq!(
            pieces.concat(),
            "https://learn.microsoft.com/en-us/credentials"
        );
    }

    #[test]
    fn a_long_document_runs_onto_a_second_page() {
        let many = (1..200)
            .map(|n| format!("Line {n} of the summary."))
            .collect::<Vec<_>>()
            .join("\n");
        let pdf = markdown_to_pdf("Long", &many);

        assert!(pdf.starts_with(b"%PDF-1.7"));
        assert_eq!(
            String::from_utf8_lossy(&pdf)
                .matches("/Type /Page ")
                .count(),
            String::from_utf8_lossy(&pdf)
                .split("/Count ")
                .nth(1)
                .and_then(|rest| rest.split(' ').next())
                .and_then(|count| count.trim_end_matches(">>").trim().parse::<usize>().ok())
                .expect("a page count"),
        );
    }

    /// The claim this module cannot make on its own: that the file opens, that the text is in it,
    /// and that no line runs past the margin. `pdfium` is already in this app to read PDFs, so it
    /// is the one asked — the writer does not get to mark its own paper.
    #[test]
    fn pdfium_reads_back_what_was_written() {
        use openexamtrainer_ingest::pdf;

        let path = std::env::temp_dir().join("openexamtrainer-typeset.pdf");
        std::fs::write(&path, markdown_to_pdf("AZ-900", SUMMARY)).expect("write");

        let document = pdf::read_file(&path, &[pdf::vendored_library_dir()]).expect("read back");
        let text: String = document
            .lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(text.contains("Zusammenfassung"), "{text}");
        // The umlauts and the em dash survive WinAnsi, which is the encoding the font declares.
        assert!(text.contains("Größe"), "{text}");
        assert!(text.contains("Ausfälle"), "{text}");
        assert!(text.contains('—'), "{text}");
        assert!(text.contains("Blob Storage"), "{text}");

        let page_width = document.pages[0].width;
        for line in &document.lines {
            assert!(
                line.x + line.width <= page_width - MARGIN + 1.0,
                "{:?} ends at {} on a page {page_width} wide",
                line.text,
                line.x + line.width
            );
            assert!(
                line.x >= MARGIN - 1.0,
                "{:?} starts at {}",
                line.text,
                line.x
            );
        }

        std::fs::remove_file(&path).ok();
    }
}

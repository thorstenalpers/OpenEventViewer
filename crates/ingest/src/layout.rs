use std::collections::{HashMap, HashSet};

use crate::model::{Document, PageGeometry, TextLine};

/// One positioned character, as delivered by the PDF backend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    pub page: u16,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub ch: char,
}

#[derive(Debug, Clone, Copy)]
pub struct AssemblyConfig {
    /// Fraction of the font size two glyphs may differ in `y` and still share a line.
    pub baseline_tolerance: f32,
    /// Gap, in font sizes, above which a single space is inserted.
    pub space_gap: f32,
    /// Gap, in font sizes, above which the line is cut in two.
    ///
    /// Page furniture is typeset far to the right of the body text and frequently shares a
    /// baseline with it. Without this cut, a footer ends up inside an answer option and every
    /// later stage inherits the corruption.
    pub split_gap: f32,
}

impl Default for AssemblyConfig {
    fn default() -> Self {
        Self {
            baseline_tolerance: 0.4,
            space_gap: 0.22,
            split_gap: 2.5,
        }
    }
}

pub fn assemble_lines(glyphs: &[Glyph], config: AssemblyConfig) -> Vec<TextLine> {
    let mut by_page: HashMap<u16, Vec<Glyph>> = HashMap::new();
    for glyph in glyphs {
        if glyph.ch == '\r' || glyph.ch == '\n' {
            continue;
        }
        by_page.entry(glyph.page).or_default().push(*glyph);
    }

    let mut pages: Vec<u16> = by_page.keys().copied().collect();
    pages.sort_unstable();

    let mut lines = Vec::new();
    for page in pages {
        let mut page_glyphs = by_page.remove(&page).unwrap_or_default();
        page_glyphs.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });

        for band in group_by_baseline(&page_glyphs, config.baseline_tolerance) {
            lines.extend(split_band(&band, config));
        }
    }

    lines.sort_by(|a, b| {
        a.page
            .cmp(&b.page)
            .then(a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });
    lines
}

fn group_by_baseline(glyphs: &[Glyph], tolerance: f32) -> Vec<Vec<Glyph>> {
    let mut bands: Vec<Vec<Glyph>> = Vec::new();
    for glyph in glyphs {
        let fits = bands.last().is_some_and(|band| {
            let reference = band[0];
            (glyph.y - reference.y).abs() <= tolerance * reference.font_size.max(1.0)
        });
        if fits {
            bands.last_mut().expect("checked above").push(*glyph);
        } else {
            bands.push(vec![*glyph]);
        }
    }
    for band in &mut bands {
        band.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    }
    bands
}

fn split_band(band: &[Glyph], config: AssemblyConfig) -> Vec<TextLine> {
    let mut out = Vec::new();
    let mut current: Vec<Glyph> = Vec::new();

    for glyph in band {
        if let Some(previous) = current.last() {
            let gap = glyph.x - (previous.x + previous.width);
            let unit = previous.font_size.max(1.0);
            if gap > config.split_gap * unit {
                out.extend(finish_line(&current, config));
                current.clear();
            }
        }
        current.push(*glyph);
    }
    out.extend(finish_line(&current, config));
    out
}

fn finish_line(glyphs: &[Glyph], config: AssemblyConfig) -> Option<TextLine> {
    let first = glyphs.first()?;
    let mut text = String::with_capacity(glyphs.len());
    let mut right = first.x;
    let mut top = first.y;
    let mut bottom = first.y + first.height;
    let mut font_size: f32 = 0.0;

    for (index, glyph) in glyphs.iter().enumerate() {
        if index > 0 {
            let gap = glyph.x - right;
            if gap > config.space_gap * glyph.font_size.max(1.0) && !text.ends_with(' ') {
                text.push(' ');
            }
        }
        text.push(glyph.ch);
        right = right.max(glyph.x + glyph.width);
        top = top.min(glyph.y);
        bottom = bottom.max(glyph.y + glyph.height);
        font_size = font_size.max(glyph.font_size);
    }

    let text = text.trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(TextLine {
        page: first.page,
        x: first.x,
        y: top,
        width: right - first.x,
        height: bottom - top,
        font_size,
        text,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct FurnitureConfig {
    /// Height of the top and bottom candidate zones, as a fraction of the page.
    pub margin: f32,
    /// Share of pages a repeated line must appear on to count as furniture.
    pub page_share: f32,
    /// Below this page count, repetition says nothing and only profile rules apply.
    pub min_pages: usize,
}

impl Default for FurnitureConfig {
    fn default() -> Self {
        Self {
            margin: 0.12,
            page_share: 0.6,
            min_pages: 4,
        }
    }
}

/// Removes headers, footers, watermarks and page numbers.
///
/// Only lines inside the top or bottom margin zone are candidates, so body text that happens to
/// repeat — `Answer: A` occurs on most pages of a dump — is never a candidate at all.
pub fn strip_furniture(
    document: &Document,
    extra: &[regex::Regex],
    config: FurnitureConfig,
) -> (Vec<TextLine>, usize) {
    let mut repeated: HashSet<String> = HashSet::new();
    let page_count = document.pages.len();

    if page_count >= config.min_pages {
        let mut pages_per_key: HashMap<String, HashSet<u16>> = HashMap::new();
        for line in &document.lines {
            if !in_margin(line, document.page(line.page), config.margin) {
                continue;
            }
            pages_per_key
                .entry(normalize(&line.text))
                .or_default()
                .insert(line.page);
        }
        let threshold = (page_count as f32 * config.page_share).ceil() as usize;
        repeated = pages_per_key
            .into_iter()
            .filter(|(_, pages)| pages.len() >= threshold)
            .map(|(key, _)| key)
            .collect();
    }

    let mut dropped = 0;
    let kept = document
        .lines
        .iter()
        .filter(|line| {
            let is_furniture = extra.iter().any(|re| re.is_match(&line.text))
                || (in_margin(line, document.page(line.page), config.margin)
                    && repeated.contains(&normalize(&line.text)));
            if is_furniture {
                dropped += 1;
            }
            !is_furniture
        })
        .cloned()
        .collect();
    (kept, dropped)
}

fn in_margin(line: &TextLine, page: Option<&PageGeometry>, margin: f32) -> bool {
    let Some(page) = page else { return false };
    if page.height <= 0.0 {
        return false;
    }
    let relative = line.y / page.height;
    relative <= margin || relative >= 1.0 - margin
}

/// Collapses a line to the form used for comparing it across pages: page numbers and dates vary,
/// the surrounding text does not.
fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_was_space = true;
    for ch in text.chars() {
        let mapped = if ch.is_ascii_digit() {
            '#'
        } else if ch.is_whitespace() {
            ' '
        } else {
            ch.to_ascii_lowercase()
        };
        if mapped == ' ' {
            if last_was_space {
                continue;
            }
            last_was_space = true;
        } else {
            last_was_space = false;
        }
        out.push(mapped);
    }
    out.trim().to_string()
}

/// Splits a page into columns and returns the lines in reading order.
///
/// Conservative on purpose: a column boundary is only accepted when the vertical corridor is wide,
/// central, and crossed by no line. A false positive here would interleave two questions.
pub fn reading_order(lines: &[TextLine], pages: &[PageGeometry]) -> Vec<TextLine> {
    let mut out = Vec::new();
    for page in pages {
        let mut page_lines: Vec<TextLine> = lines
            .iter()
            .filter(|l| l.page == page.index)
            .cloned()
            .collect();
        match column_split(&page_lines, page) {
            Some(boundary) => {
                let (mut left, mut right): (Vec<TextLine>, Vec<TextLine>) =
                    page_lines.into_iter().partition(|l| l.right() <= boundary);
                sort_by_position(&mut left);
                sort_by_position(&mut right);
                out.append(&mut left);
                out.append(&mut right);
            }
            None => {
                sort_by_position(&mut page_lines);
                out.append(&mut page_lines);
            }
        }
    }
    out
}

fn sort_by_position(lines: &mut [TextLine]) {
    lines.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });
}

fn column_split(lines: &[TextLine], page: &PageGeometry) -> Option<f32> {
    if lines.len() < 12 || page.width <= 0.0 {
        return None;
    }
    let lower = page.width * 0.35;
    let upper = page.width * 0.65;
    let mut candidate = lower;
    while candidate <= upper {
        let crossed = lines
            .iter()
            .any(|l| l.x < candidate - 1.0 && l.right() > candidate + 1.0);
        if !crossed {
            let left = lines.iter().filter(|l| l.right() <= candidate).count();
            let right = lines.len() - left;
            if left >= 5 && right >= 5 {
                return Some(candidate);
            }
        }
        candidate += page.width * 0.01;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph(page: u16, x: f32, y: f32, ch: char) -> Glyph {
        Glyph {
            page,
            x,
            y,
            width: 5.0,
            height: 10.0,
            font_size: 10.0,
            ch,
        }
    }

    fn word(page: u16, x: f32, y: f32, text: &str) -> Vec<Glyph> {
        text.chars()
            .enumerate()
            .map(|(i, ch)| glyph(page, x + i as f32 * 5.0, y, ch))
            .collect()
    }

    fn page(index: u16) -> PageGeometry {
        PageGeometry {
            index,
            width: 600.0,
            height: 800.0,
        }
    }

    #[test]
    fn a_far_right_run_on_the_same_baseline_becomes_its_own_line() {
        let mut glyphs = word(1, 50.0, 700.0, "A.Mastered");
        glyphs.extend(word(1, 400.0, 700.0, "visit"));

        let lines = assemble_lines(&glyphs, AssemblyConfig::default());

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "A.Mastered");
        assert_eq!(lines[1].text, "visit");
    }

    #[test]
    fn a_moderate_gap_becomes_a_space_not_a_split() {
        let mut glyphs = word(1, 50.0, 700.0, "Answer:");
        glyphs.extend(word(1, 90.0, 700.0, "A"));

        let lines = assemble_lines(&glyphs, AssemblyConfig::default());

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "Answer: A");
    }

    #[test]
    fn repeated_footers_are_dropped_and_repeated_body_text_is_not() {
        let mut lines = Vec::new();
        for page_index in 1..=5u16 {
            lines.push(TextLine {
                page: page_index,
                x: 50.0,
                y: 770.0,
                width: 200.0,
                height: 10.0,
                font_size: 8.0,
                text: format!("visit - https://www.certshared.com page {page_index}"),
            });
            lines.push(TextLine {
                page: page_index,
                x: 50.0,
                y: 400.0,
                width: 60.0,
                height: 10.0,
                font_size: 10.0,
                text: "Answer: A".to_string(),
            });
        }
        let document = Document {
            pages: (1..=5).map(page).collect(),
            lines,
        };

        let (kept, dropped) = strip_furniture(&document, &[], FurnitureConfig::default());

        assert_eq!(dropped, 5);
        assert_eq!(kept.len(), 5);
        assert!(kept.iter().all(|l| l.text == "Answer: A"));
    }

    #[test]
    fn a_single_column_page_keeps_its_vertical_order() {
        let pages = vec![page(1)];
        let lines: Vec<TextLine> = (0..15)
            .map(|i| TextLine {
                page: 1,
                x: 50.0,
                y: 100.0 + i as f32 * 12.0,
                width: 400.0,
                height: 10.0,
                font_size: 10.0,
                text: format!("line {i}"),
            })
            .collect();

        let ordered = reading_order(&lines, &pages);

        assert_eq!(ordered.first().unwrap().text, "line 0");
        assert_eq!(ordered.last().unwrap().text, "line 14");
    }
}

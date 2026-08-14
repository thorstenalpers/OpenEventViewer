use std::path::Path;

use image::ImageEncoder;
use pdfium_render::prelude::*;
use sha2::{Digest, Sha256};

use crate::model::{Asset, Document, ExtractionReport, FigureBand};
use crate::IngestError;
use crate::{layout, pdf};

/// Render resolution for a captured figure. 200 dpi keeps the small type inside an Azure portal
/// screenshot readable without turning a 12-question binder into a hundred megabytes.
const DPI: f32 = 200.0;
const POINTS_PER_INCH: f32 = 72.0;

/// A pixel has to be visibly darker than the paper to count. Anti-aliasing and JPEG ringing leave a
/// scatter of near-white pixels behind in a genuinely empty region, so "not pure white" is not the
/// test.
const INK_THRESHOLD: u8 = 235;
const MIN_INK_RATIO: f32 = 0.002;

/// A ratio over a sliver of unmasked pixels is noise, not a measurement: mask most of a band and
/// two stray anti-aliased dots read as 100 % ink. Both floors are absolute, in pixels at `DPI`.
const MIN_MEASURED_PIXELS: u32 = 40_000;
const MIN_INK_PIXELS: u32 = 400;

/// A hole shorter than this is paragraph spacing or a page break, not a place a figure was.
const MIN_BAND_POINTS: f32 = 20.0;

/// Glyphs overshoot their reported box by a hair — descenders, italic overhang, anti-aliasing.
const TEXT_PADDING_POINTS: f32 = 2.0;

/// Breathing room around the captured figure, in pixels at `DPI`.
const CROP_PADDING: u32 = 12;

/// How far past its text a running header reaches — the rule drawn under it.
const FURNITURE_PADDING_POINTS: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Mask {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Bounds {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
}

/// Fills in `figures` for every question that has a band, and returns the assets they point at.
///
/// The band is rasterised rather than lifted out as an embedded image object on purpose: an exam
/// figure is as often a vector-drawn table as it is a screenshot, and the rendered region is what
/// the reader actually saw.
///
/// `document` is the *unfiltered* line set — including the page furniture the extractor dropped.
/// A figure is ink that is not text, so every known line is masked out before the band is measured;
/// without that, a footer sitting between the stem and the options is captured as a diagram.
pub fn capture(
    source: &Path,
    document: &Document,
    report: &mut ExtractionReport,
) -> Result<Vec<Asset>, IngestError> {
    let bands: Vec<(usize, FigureBand)> = report
        .questions
        .iter()
        .enumerate()
        .filter_map(|(index, question)| question.figure_band.map(|band| (index, band)))
        .collect();
    if bands.is_empty() {
        return Ok(Vec::new());
    }

    let mut assets: Vec<Asset> = Vec::new();
    pdf::with_document(source, &[], |pdf_document| {
        for (index, band) in &bands {
            let Ok(page) = pdf_document.pages().get(band.page.saturating_sub(1).into()) else {
                continue;
            };
            let Some(png) = render_band(&page, band, document) else {
                continue;
            };

            let hash = format!("{:x}", Sha256::digest(&png));
            if !assets.iter().any(|asset| asset.hash == hash) {
                assets.push(Asset {
                    hash: hash.clone(),
                    png,
                });
            }
            report.questions[*index].figures.push(hash);
        }
        Ok(())
    })?;

    // A recovered figure is the missing half of an `A. Mastered / B. Not Mastered` question: with
    // the picture back and the answer key already read out of the explanation, the question is
    // answerable and belongs in scored sessions. Re-scoring rather than adjusting the number by
    // hand keeps one definition of confidence.
    let mut recovered = false;
    for question in &mut report.questions {
        if !question.figures.is_empty() && question.needs_source {
            question.needs_source = false;
            recovered = true;
        }
    }
    if recovered {
        crate::confidence::score(&mut report.questions);
    }

    Ok(assets)
}

fn render_band(page: &PdfPage<'_>, band: &FigureBand, document: &Document) -> Option<Vec<u8>> {
    // A band is derived from two text lines and nothing stops it running off the sheet — a stem
    // whose options continue overleaf produces a "gap" that is really a page break. Clamp first,
    // then check what is left is big enough to have held a figure.
    let page_points = page.height().value;
    let band_top = band.top.clamp(0.0, page_points);
    let band_bottom = band.bottom.unwrap_or(page_points).clamp(0.0, page_points);
    if band_bottom - band_top < MIN_BAND_POINTS {
        return None;
    }
    let band = FigureBand {
        page: band.page,
        top: band_top,
        bottom: Some(band_bottom),
    };

    let scale = DPI / POINTS_PER_INCH;
    let bitmap = page
        .render(
            (page.width().value * scale) as i32,
            (page.height().value * scale) as i32,
            None,
        )
        .ok()?;

    // `as_image` rather than the raw byte buffer: Pdfium hands back BGRA with its own stride, and
    // reading that as tightly packed RGBA would swap red and blue in every captured screenshot —
    // wrong in a way that looks deliberate rather than broken.
    let rendered = bitmap.as_image().ok()?.to_rgba8();
    let full_width = rendered.width();
    let full_height = rendered.height();

    let top = ((band.top * scale) as u32).min(full_height);
    let bottom = ((band.bottom.unwrap_or(page_points) * scale) as u32).min(full_height);
    if bottom <= top + 1 {
        return None;
    }

    let furniture = furniture_masks(document, &band, scale, top, full_width);
    let text = text_masks(document, &band, scale, top, full_width);
    let band_image =
        image::imageops::crop_imm(&rendered, 0, top, full_width, bottom - top).to_image();
    let bounds = ink_bounds(&band_image, &furniture, &text)?;
    let cropped = image::imageops::crop_imm(
        &band_image,
        bounds.left,
        bounds.top,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
    )
    .to_image();

    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            cropped.as_raw(),
            cropped.width(),
            cropped.height(),
            image::ExtendedColorType::Rgba8,
        )
        .ok()?;
    Some(png)
}

/// The page's header and footer strips, full width.
///
/// A vendor's running header is a logo and a rule as much as it is a sentence, and neither of those
/// carries a `TextLine` to mask — so the strip has to be geometry. Its depth comes from where the
/// header's own lines actually end rather than from the extractor's 12 % candidate zone: that zone
/// is deliberately generous because it only ever *nominates* repeated lines, and using it as an ink
/// mask would cut the top off every figure that starts high on a page.
fn furniture_masks(
    document: &Document,
    band: &FigureBand,
    scale: f32,
    crop_top: u32,
    width: u32,
) -> Vec<Mask> {
    let Some(page) = document.page(band.page) else {
        return Vec::new();
    };

    let margin = page.height * layout::FurnitureConfig::default().margin;
    let on_page = || document.lines.iter().filter(|l| l.page == band.page);
    let header_ends = on_page()
        .filter(|l| l.y < margin)
        .map(|l| l.y + l.height)
        .fold(0.0f32, f32::max);
    let footer_starts = on_page()
        .filter(|l| l.y + l.height > page.height - margin)
        .map(|l| l.y)
        .fold(page.height, f32::min);

    let mut strips = Vec::new();
    if header_ends > 0.0 {
        strips.push((0.0, header_ends + FURNITURE_PADDING_POINTS));
    }
    if footer_starts < page.height {
        strips.push((footer_starts - FURNITURE_PADDING_POINTS, page.height));
    }

    strips
        .into_iter()
        .filter_map(|(top, bottom)| {
            let bottom = (bottom * scale) as u32;
            (bottom > crop_top).then(|| Mask {
                top: ((top.max(0.0) * scale) as u32).saturating_sub(crop_top),
                bottom: bottom - crop_top,
                left: 0,
                right: width,
            })
        })
        .collect()
}

/// Every known text line on the band's page, in the cropped image's own pixel coordinates.
fn text_masks(
    document: &Document,
    band: &FigureBand,
    scale: f32,
    crop_top: u32,
    width: u32,
) -> Vec<Mask> {
    document
        .lines
        .iter()
        .filter(|line| line.page == band.page)
        .filter_map(|line| {
            let top = ((line.y - TEXT_PADDING_POINTS) * scale).max(0.0) as u32;
            let bottom = ((line.y + line.height + TEXT_PADDING_POINTS) * scale) as u32;
            if bottom <= crop_top {
                return None;
            }
            Some(Mask {
                top: top.saturating_sub(crop_top),
                bottom: bottom - crop_top,
                left: ((line.x - TEXT_PADDING_POINTS) * scale).max(0.0) as u32,
                right: (((line.right() + TEXT_PADDING_POINTS) * scale) as u32).min(width),
            })
        })
        .collect()
}

/// The box to crop the figure to, or `None` when the band holds no figure.
///
/// The two mask sets answer two different questions, which is why they are not one list:
///
/// * `furniture` is not part of the question at all. It is skipped entirely — a band that reaches
///   the top of a page contains the vendor's header, and a logo or a rule carries no `TextLine` to
///   mask, so only the zone keeps the advertisement out of the captured image.
/// * `text` is the question, already extracted and displayed above the figure. It must not count
///   toward "is there a diagram here", or a footer between the stem and the options would read as
///   one — but it is still cropped *in*, because a heading like `Answer Area` and the sentence a
///   dropdown sits inside are what make the picture legible.
fn ink_bounds(image: &image::RgbaImage, furniture: &[Mask], text: &[Mask]) -> Option<Bounds> {
    let covers = |masks: &[Mask], x: u32, y: u32| {
        masks
            .iter()
            .any(|m| y >= m.top && y < m.bottom && x >= m.left && x < m.right)
    };

    let mut measured = 0u32;
    let mut inked = 0u32;
    let (mut left, mut top) = (u32::MAX, u32::MAX);
    let (mut right, mut bottom) = (0u32, 0u32);

    for (x, y, pixel) in image.enumerate_pixels() {
        if covers(furniture, x, y) {
            continue;
        }
        // Transparent pixels are paper, not ink, whatever their colour channels say.
        let is_ink = pixel.0[3] > 16 && pixel.0[..3].iter().any(|c| *c < INK_THRESHOLD);
        if is_ink {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
        if covers(text, x, y) {
            continue;
        }
        measured += 1;
        if is_ink {
            inked += 1;
        }
    }

    let enough = measured >= MIN_MEASURED_PIXELS
        && inked >= MIN_INK_PIXELS
        && inked as f32 / measured as f32 >= MIN_INK_RATIO;
    if !enough {
        return None;
    }

    Some(Bounds {
        left: left.saturating_sub(CROP_PADDING),
        top: top.saturating_sub(CROP_PADDING),
        right: (right + CROP_PADDING).min(image.width()),
        bottom: (bottom + CROP_PADDING).min(image.height()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paper(width: u32, height: u32) -> image::RgbaImage {
        image::RgbaImage::from_pixel(width, height, image::Rgba([255, 255, 255, 255]))
    }

    #[test]
    fn a_blank_band_carries_no_ink_and_a_dark_line_does() {
        let mut band = paper(300, 300);
        assert_eq!(ink_bounds(&band, &[], &[]), None);

        for x in 0..300 {
            for y in 40..45 {
                band.put_pixel(x, y, image::Rgba([20, 20, 20, 255]));
            }
        }
        assert!(ink_bounds(&band, &[], &[]).is_some());
    }

    /// The captured PNG is the figure, not the page it sat on: whitespace above and below the ink
    /// is what makes a recovered diagram unreadable in a drill card.
    #[test]
    fn the_capture_is_cropped_to_the_ink_with_a_margin() {
        let mut band = paper(300, 300);
        for x in 100..160 {
            for y in 40..80 {
                band.put_pixel(x, y, image::Rgba([20, 20, 20, 255]));
            }
        }

        let bounds = ink_bounds(&band, &[], &[]).expect("ink");

        assert_eq!(
            bounds,
            Bounds {
                left: 100 - CROP_PADDING,
                top: 40 - CROP_PADDING,
                right: 160 + CROP_PADDING,
                bottom: 80 + CROP_PADDING,
            }
        );
    }

    /// A figure at the top of a page shares the band with the running header. Cropping to unmasked
    /// ink is what keeps the vendor's advertisement out of the captured image.
    #[test]
    fn a_masked_header_is_cropped_away_rather_than_merely_ignored() {
        let mut band = paper(300, 300);
        for x in 0..300 {
            for y in 0..20 {
                band.put_pixel(x, y, image::Rgba([200, 20, 20, 255]));
            }
        }
        for x in 100..200 {
            for y in 150..200 {
                band.put_pixel(x, y, image::Rgba([20, 20, 20, 255]));
            }
        }

        let header = Mask {
            top: 0,
            bottom: 24,
            left: 0,
            right: 300,
        };

        assert_eq!(
            ink_bounds(&band, &[header], &[]).expect("ink").top,
            150 - CROP_PADDING
        );
    }

    /// The footer that sits between a stem and its options is text, not a diagram. Masking it is
    /// the difference between capturing a figure and capturing an advertisement.
    #[test]
    fn ink_inside_a_masked_text_line_does_not_count() {
        let mut band = paper(300, 300);
        for x in 10..290 {
            for y in 40..45 {
                band.put_pixel(x, y, image::Rgba([20, 20, 20, 255]));
            }
        }

        let mask = Mask {
            top: 38,
            bottom: 47,
            left: 8,
            right: 292,
        };

        assert!(ink_bounds(&band, &[], &[]).is_some());
        assert_eq!(ink_bounds(&band, &[], &[mask]), None);
    }

    #[test]
    fn near_white_scatter_is_paper_not_a_figure() {
        let mut speckled = paper(300, 300);
        for x in 0..300 {
            speckled.put_pixel(x, 10, image::Rgba([248, 248, 248, 255]));
        }

        assert_eq!(ink_bounds(&speckled, &[], &[]), None);
    }

    #[test]
    fn a_fully_transparent_band_is_paper() {
        let transparent = image::RgbaImage::from_pixel(300, 300, image::Rgba([0, 0, 0, 0]));

        assert_eq!(ink_bounds(&transparent, &[], &[]), None);
    }

    #[test]
    fn a_band_masked_end_to_end_reports_no_ink_rather_than_dividing_by_zero() {
        let band = paper(300, 300);
        let everything = Mask {
            top: 0,
            bottom: 300,
            left: 0,
            right: 300,
        };

        assert_eq!(ink_bounds(&band, &[], &[everything]), None);
    }
}

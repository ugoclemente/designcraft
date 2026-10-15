//! Optical spacing derived from glyph contours, independent of font kerning tables.
//! Equalize the nearer three quarters of the facing white-space samples to the
//! font's own `nn` rhythm. The closest samples bound tightening so ink cannot
//! collide. Spaces, joining scripts and marks retain the shaper's advances.
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use designcraft_fonts::{FontFace, ScopedFonts};
use designcraft_geom::{PathEl, Point, Shape, kurbo};

const SAMPLES: usize = 128;
type Profile = [Option<(f64, f64)>; SAMPLES];
type Profiles = HashMap<(u32, u32), Arc<Profile>>;

fn profile(db: &ScopedFonts<'_>, face: &FontFace, gid: u32) -> Arc<Profile> {
    static CACHE: OnceLock<Mutex<Profiles>> = OnceLock::new();
    let cache = CACHE.get_or_init(Mutex::default);
    let key = (face.id(), gid);
    if let Some(p) = cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(&key).cloned() {
        return p;
    }
    let mut result: Profile = [None; SAMPLES];
    let outline = db.outline(face, gid);
    let tolerance = face.upem / 1024.0;
    let bounds = outline.bounding_box();
    if face.upem.is_finite()
        && tolerance > 0.0
        && outline.elements().len() <= 4096
        && [bounds.x0, bounds.y0, bounds.x1, bounds.y1].iter().all(|v| v.is_finite() && v.abs() <= face.upem * 16.0)
    {
        let mut last = Point::ZERO;
        let mut start = last;
        let top = -face.ascent.max(face.cap_height);
        let height = face.descent - top;
        let mut edge = |a: Point, b: Point| {
            if ![a.x, a.y, b.x, b.y].iter().all(|v| v.is_finite()) {
                return;
            }
            for (i, sample) in result.iter_mut().enumerate() {
                let y = top + (i as f64 + 0.5) * height / SAMPLES as f64;
                if (a.y <= y && y < b.y) || (b.y <= y && y < a.y) {
                    let x = a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y);
                    *sample = Some(sample.map_or((x, x), |(left, right)| (left.min(x), right.max(x))));
                }
            }
        };
        kurbo::flatten(outline.iter(), tolerance, |element| match element {
            PathEl::MoveTo(p) => {
                last = p;
                start = p;
            }
            PathEl::LineTo(p) => {
                edge(last, p);
                last = p;
            }
            PathEl::ClosePath => {
                edge(last, start);
                last = start;
            }
            _ => {}
        });
    }
    let result = Arc::new(result);
    let mut cache = cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if cache.len() >= 4096 {
        cache.clear();
    }
    cache.insert(key, result.clone());
    result
}

fn gap(a: &Profile, b: &Profile, advance: f64) -> Option<(f64, f64)> {
    let mut gaps: Vec<f64> = a
        .iter()
        .zip(b)
        .filter_map(|(a, b)| {
            let (_, right) = (*a)?;
            let (left, _) = (*b)?;
            let gap = advance + left - right;
            gap.is_finite().then_some(gap)
        })
        .collect();
    gaps.sort_by(f64::total_cmp);
    let minimum = *gaps.first()?;
    let count = (gaps.len() * 3 / 4).max(1);
    Some((gaps.iter().take(count).sum::<f64>() / count as f64, minimum))
}

pub(crate) fn latin(c: char) -> bool {
    c.is_alphabetic() && matches!(c as u32, 0x20..=0x024F | 0x1E00..=0x1EFF)
}

/// A replacement advance in font units. Unsupported geometry retains shaping.
pub(crate) fn advance(db: &ScopedFonts<'_>, face: &FontFace, a: u32, b: u32) -> Option<f64> {
    let n = face.glyph_for('n');
    if a == 0 || b == 0 || n == 0 || ![face.upem, face.ascent, face.descent, face.cap_height].iter().all(|v| v.is_finite()) || face.upem <= 0.0 {
        return None;
    }
    let reference = profile(db, face, n);
    let (target, _) = gap(&reference, &reference, face.advance(n))?;
    let (measured, minimum) = gap(&profile(db, face, a), &profile(db, face, b), face.advance(a))?;
    let epsilon = face.upem / 1024.0;
    let lower = (-minimum + epsilon).max(-face.upem / 4.0);
    let upper = face.upem / 4.0;
    if lower > upper {
        return None;
    }
    let adjustment = (target - measured).clamp(lower, upper);
    let advance = face.advance(a) + adjustment;
    (advance.is_finite() && advance > 0.0).then_some(advance)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn facing_ink_stays_separate_and_repeated_stems_retain_the_fonts_rhythm() {
        let db = designcraft_fonts::FontDb::global().scoped(0);
        for family in ["Source Serif 4", "Source Sans 3"] {
            let face = db.face(family, "Regular");
            let n = face.glyph_for('n');
            assert!((advance(&db, &face, n, n).unwrap() - face.advance(n)).abs() < 1e-9);
            let mut changed = false;
            for a in "AVToPefinm".chars() {
                for b in "AVToPefinm".chars() {
                    let (a, b) = (face.glyph_for(a), face.glyph_for(b));
                    let Some(width) = advance(&db, &face, a, b) else { continue };
                    changed |= (width - face.advance(a)).abs() > face.upem / 1000.0;
                    let (_, closest) = gap(&profile(&db, &face, a), &profile(&db, &face, b), width).unwrap();
                    assert!(closest >= face.upem / 1024.0 - 1e-7, "{family}: {a} {b}: {closest}");
                }
            }
            assert!(changed);
        }
        assert!(advance(&db, &db.face("Source Serif 4", "Regular"), 0, 1).is_none());
        assert!(!latin('م') && !latin('一') && !latin('\u{0301}') && !latin(' '));
    }
}

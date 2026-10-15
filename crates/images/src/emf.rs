//! Portable raster fallback for bitmap-only enhanced metafiles. Vector records
//! are rejected rather than silently omitted. MS-EMF 2.3.1.7 defines the DIB
//! offsets; SRCPAINT followed by SRCAND encodes binary transparency.

use image::RgbaImage;

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at.checked_add(4)?)?.try_into().ok()?))
}

pub fn is_emf(b: &[u8]) -> bool {
    u32_at(b, 0) == Some(1) && b.get(40..44) == Some(b" EMF")
}

fn dib(record: &[u8]) -> Option<RgbaImage> {
    let section = |at| {
        let start = usize::try_from(u32_at(record, at)?).ok()?;
        let len = usize::try_from(u32_at(record, at + 4)?).ok()?;
        (start >= 80).then_some(())?;
        record.get(start..start.checked_add(len)?)
    };
    let header = section(48)?;
    let bits = section(56)?;
    if u32_at(header, 0)? != 40 || u32_at(header, 16)? != 0 || u32_at(record, 64)? != 0 {
        return None;
    }
    let width = u32_at(header, 4)?;
    let height = i32::from_le_bytes(header.get(8..12)?.try_into().ok()?).unsigned_abs();
    let pixels = width.checked_mul(height)?;
    if width == 0 || height == 0 || width > 8192 || height > 8192 || pixels > 16_777_216 {
        return None;
    }
    // Only complete, unflipped source bitmaps with no crop or resampling.
    if u32_at(record, 32)? != 0
        || u32_at(record, 36)? != 0
        || u32_at(record, 40)? != width
        || u32_at(record, 44)? != height
        || u32_at(record, 72)? != width
        || u32_at(record, 76)? != height
    {
        return None;
    }
    let length = 14usize.checked_add(header.len())?.checked_add(bits.len())?;
    if length > 128 * 1024 * 1024 {
        return None;
    }
    let mut bmp = Vec::new();
    bmp.try_reserve_exact(length).ok()?;
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&u32::try_from(length).ok()?.to_le_bytes());
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&u32::try_from(14 + header.len()).ok()?.to_le_bytes());
    bmp.extend_from_slice(header);
    bmp.extend_from_slice(bits);
    Some(image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp).ok()?.to_rgba8())
}

pub fn decode(bytes: &[u8]) -> Option<RgbaImage> {
    if !is_emf(bytes) || bytes.len() > 128 * 1024 * 1024 {
        return None;
    }
    let mut offset = 0usize;
    let mut count = 0u32;
    let mut mask: Option<(Vec<u8>, RgbaImage)> = None;
    let mut image = None;
    loop {
        let kind = u32_at(bytes, offset)?;
        let size = usize::try_from(u32_at(bytes, offset.checked_add(4)?)?).ok()?;
        if size < 8 || size % 4 != 0 || count >= 100_000 {
            return None;
        }
        let end = offset.checked_add(size)?;
        let record = bytes.get(offset..end)?;
        count += 1;
        match kind {
            1 if offset == 0 => {
                if size < 88 || usize::try_from(u32_at(record, 48)?).ok()? != bytes.len() {
                    return None;
                }
            }
            14 => return (end == bytes.len() && mask.is_none() && u32_at(bytes, 52)? == count).then_some(image)?,
            // Comments contain the editable vector source or an EMF+ alternative;
            // the complete GDI bitmap fallback below is what we display.
            70 | 21 | 33 | 34 | 98 => {}
            76 if u32_at(record, 40)? == 0x00aa0029 => {} // DSTCOPY: no change.
            81 if image.is_none() => {
                let raster = dib(record)?;
                let geometry = record.get(24..48)?.to_vec();
                match u32_at(record, 68)? {
                    0x00ee0086 if mask.is_none() => mask = Some((geometry, raster)),
                    0x008800c6 => {
                        let (at, mask_image) = mask.take()?;
                        if at != geometry || mask_image.dimensions() != raster.dimensions() {
                            return None;
                        }
                        let mut result = raster;
                        for (pixel, mask_pixel) in result.pixels_mut().zip(mask_image.pixels()) {
                            match mask_pixel.0 {
                                [255, 255, 255, _] => pixel.0[3] = 255,
                                [0, 0, 0, _] if pixel.0[..3] == [255, 255, 255] => pixel.0 = [0; 4],
                                [0, 0, 0, _] if pixel.0[..3] == [0, 0, 0] => pixel.0[3] = 255,
                                _ => return None,
                            }
                        }
                        image = Some(result);
                    }
                    0x00cc0020 if mask.is_none() => image = Some(raster), // SRCCOPY.
                    _ => return None,
                }
            }
            _ => return None,
        }
        offset = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(b: &mut [u8], at: usize, value: u32) {
        b[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn record(kind: u32, size: usize) -> Vec<u8> {
        let mut b = vec![0; size];
        set(&mut b, 0, kind);
        set(&mut b, 4, size as u32);
        b
    }
    fn bitmap(rop: u32, color: [u8; 8]) -> Vec<u8> {
        let mut b = record(81, 128);
        for (at, v) in
            [(40, 2), (44, 1), (48, 80), (52, 40), (56, 120), (60, 8), (68, rop), (72, 2), (76, 1), (80, 40), (84, 2), (88, 1), (92, 0x00180001)]
        {
            set(&mut b, at, v);
        }
        b[120..128].copy_from_slice(&color);
        b
    }
    fn sample() -> Vec<u8> {
        let mut b = record(1, 108);
        b[40..44].copy_from_slice(b" EMF");
        b.extend(bitmap(0x00ee0086, [255, 255, 255, 0, 0, 0, 0, 0]));
        b.extend(bitmap(0x008800c6, [0, 128, 0, 255, 255, 255, 0, 0]));
        b.extend(record(14, 20));
        let len = b.len() as u32;
        set(&mut b, 48, len);
        set(&mut b, 52, 4);
        b
    }
    #[test]
    fn masked_bitmap_fallback_preserves_color_and_transparency() {
        let b = sample();
        let img = decode(&b).unwrap();
        assert_eq!(img.dimensions(), (2, 1));
        assert_eq!(img.get_pixel(0, 0).0, [0, 128, 0, 255]);
        assert_eq!(img.get_pixel(1, 0).0, [0, 0, 0, 0]);
        assert_eq!(crate::mime(&b), "image/emf");
    }
    #[test]
    fn malformed_or_unsupported_emf_never_becomes_a_partial_picture() {
        let b = sample();
        for end in 0..b.len() {
            assert!(decode(&b[..end]).is_none());
        }
        for (at, value) in
            [(112, 0), (108, 3), (48, u32::MAX), (52, u32::MAX), (108 + 48, u32::MAX), (108 + 84, 100_000), (108 + 68, 0), (108 + 72, 3)]
        {
            let mut bad = b.clone();
            set(&mut bad, at, value);
            assert!(decode(&bad).is_none(), "offset {at}");
        }
    }
}

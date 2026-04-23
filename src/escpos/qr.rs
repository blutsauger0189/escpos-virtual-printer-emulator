//! QR code rendering helper.
//!
//! Converts an ESC/POS `GS ( k` QR print request into a 1-bit-per-pixel
//! packed bitmap that is layout-compatible with [`ReceiptLine::Bitmap`]
//! (row-major, MSB = leftmost pixel).
//!
//! The caller provides:
//! - the raw data bytes to encode,
//! - the module size in dots (as set by `GS ( k fn=67`),
//! - the error-correction level (as set by `GS ( k fn=69`).

use qrcode::{EcLevel, QrCode};

/// Map the ESC/POS error-correction byte (`48..=51`) to [`EcLevel`].
pub fn ec_level_from_byte(n: u8) -> EcLevel {
    match n {
        48 => EcLevel::L,
        49 => EcLevel::M,
        50 => EcLevel::Q,
        51 => EcLevel::H,
        _ => EcLevel::M,
    }
}

/// Render a QR code to a packed 1bpp bitmap.
///
/// Returns `(width_bytes, height_px, data)` where `width_bytes * 8` is the
/// padded pixel width (rounded up to the next multiple of 8). Returns `None`
/// if the data cannot be encoded (e.g. exceeds QR capacity).
pub fn render_qr(
    data: &[u8],
    module_size: u8,
    ec_level: EcLevel,
) -> Option<(u16, u16, Vec<u8>)> {
    let code = QrCode::with_error_correction_level(data, ec_level).ok()?;
    let modules = code.width();
    let module_px = module_size.max(1) as usize;
    let pixel_side = modules * module_px;

    let width_bytes = (pixel_side + 7) / 8;
    let height_px = pixel_side;

    let colors = code.to_colors();
    let mut packed = vec![0u8; width_bytes * height_px];

    for my in 0..modules {
        for mx in 0..modules {
            if colors[my * modules + mx] != qrcode::Color::Dark {
                continue;
            }
            for dy in 0..module_px {
                for dx in 0..module_px {
                    let px = mx * module_px + dx;
                    let py = my * module_px + dy;
                    let byte_idx = py * width_bytes + px / 8;
                    let bit_idx = 7 - (px % 8);
                    packed[byte_idx] |= 1 << bit_idx;
                }
            }
        }
    }

    Some((width_bytes as u16, height_px as u16, packed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_simple_payload() {
        let (wb, h, data) = render_qr(b"hello", 3, EcLevel::M).unwrap();
        // Version 1 QR (smallest) is 21x21 modules. With module_size=3 ⇒ 63 px.
        // Padded width_bytes = ceil(63/8) = 8 ⇒ 64 px logical.
        assert_eq!(wb, 8);
        assert_eq!(h, 63);
        assert_eq!(data.len(), wb as usize * h as usize);
    }

    #[test]
    fn module_size_scales_output() {
        let (_, h1, _) = render_qr(b"x", 1, EcLevel::L).unwrap();
        let (_, h4, _) = render_qr(b"x", 4, EcLevel::L).unwrap();
        assert_eq!(h4, h1 * 4);
    }

    #[test]
    fn module_size_zero_is_treated_as_one() {
        let (_, h0, _) = render_qr(b"x", 0, EcLevel::L).unwrap();
        let (_, h1, _) = render_qr(b"x", 1, EcLevel::L).unwrap();
        assert_eq!(h0, h1);
    }

    #[test]
    fn ec_level_byte_mapping() {
        assert!(matches!(ec_level_from_byte(48), EcLevel::L));
        assert!(matches!(ec_level_from_byte(49), EcLevel::M));
        assert!(matches!(ec_level_from_byte(50), EcLevel::Q));
        assert!(matches!(ec_level_from_byte(51), EcLevel::H));
        assert!(matches!(ec_level_from_byte(99), EcLevel::M));
    }

    #[test]
    fn has_dark_pixels() {
        let (_, _, data) = render_qr(b"https://example.com", 3, EcLevel::M).unwrap();
        let dark_count: usize = data.iter().map(|b| b.count_ones() as usize).sum();
        assert!(dark_count > 0, "rendered QR should contain dark pixels");
    }
}

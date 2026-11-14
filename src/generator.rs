//! QR rendering + encoding into WebP / SVG / AVIF.
//!
//! This module contains the core rendering pipeline:
//! 1. `QrCode::with_error_correction_level(text, ec_level)` — encode text into a QR module matrix
//! 2. `render_modules()` — convert the bool grid to RGBA pixels (black/white)
//! 3. `scale_nearest()` — nearest-neighbor scale to the requested output size
//! 4. Format-specific encoder — write WebP, SVG, or AVIF bytes

use crate::Format;
use qrcode::Color;
use qrcode::EcLevel;
use qrcode::QrCode;
use ravif::{Encoder, Img, RGBA8};

/// Maximum input size in bytes to prevent accidentally encoding huge files.
///
/// QR codes have a hard capacity limit: at Version 40 (the largest size),
/// Error Correction Level L, they can store at most ~2,953 bytes of binary
/// data. Anything larger cannot physically fit in a QR code, so we reject it
/// early with a clear error message rather than letting the user wait for
/// an encoding failure.
const MAX_INPUT_SIZE: usize = 2000;

/// Minimum allowed output size in pixels.
///
/// Anything smaller than 8 pixels per side is practically unscannable.
const MIN_OUTPUT_SIZE: u32 = 8;

/// Maximum allowed output size in pixels.
///
/// Prevents absurd memory allocation from a typo like `-s 999999999`.
const MAX_OUTPUT_SIZE: u32 = 8192;

/// Render the QR code into an RGBA pixel buffer (one pixel per module).
///
/// Each QR module (the black/white square cells) becomes one RGBA pixel:
/// - Dark modules  -> black (0, 0, 0, 255)
/// - Light modules -> white (255, 255, 255, 255)
///
/// Returns a flat `Vec` of `RGBA8` values in row-major order.
fn render_modules(qr: &QrCode) -> Vec<RGBA8> {
    let n = qr.width();
    let mut pixels = Vec::with_capacity(n * n);
    for y in 0..n {
        for x in 0..n {
            let (r, g, b, a) = match qr[(x, y)] {
                Color::Dark => (0, 0, 0, 255),
                Color::Light => (255, 255, 255, 255),
            };
            pixels.push(RGBA8::new(r, g, b, a));
        }
    }
    pixels
}

/// Nearest-neighbor scale an RGBA buffer from `src` x `src` to `dst` x `dst`.
///
/// Nearest-neighbor sampling picks the closest source pixel for each
/// destination pixel. This preserves the sharp, blocky edges of QR codes
/// (no blurring or interpolation), which is critical for scannability.
///
/// # Algorithm
/// For each destination pixel (dx, dy), compute the corresponding source
/// pixel: `sy = (dy * src) / dst`, `sx = (dx * src) / dst`.
fn scale_nearest(pixels: &[RGBA8], src: u32, dst: u32) -> Vec<RGBA8> {
    let mut out = Vec::with_capacity((dst * dst) as usize);
    for dy in 0..dst {
        let sy = (dy * src) / dst;
        for dx in 0..dst {
            let sx = (dx * src) / dst;
            out.push(pixels[(sy * src + sx) as usize]);
        }
    }
    out
}

/// Generate QR code image bytes for `text`, scaled to `size` x `size` pixels,
/// using the given error correction level.
///
/// This is the central pipeline function. It:
/// 1. Validates input size (rejects > 2,000 bytes)
/// 2. Validates output size bounds (8–8192 pixels)
/// 3. Encodes text into a QR code matrix with the given EC level
/// 4. Renders modules to RGBA pixels
/// 5. Scales to the requested output size
/// 6. Encodes to the requested format (WebP / SVG / AVIF)
///
/// Returns `Err(String)` on oversized input, invalid size, or encoding failure.
pub(super) fn generate_qr(
    text: &str,
    size: u32,
    format: Format,
    ec_level: EcLevel,
) -> Result<Vec<u8>, String> {
    // Guard against absurdly large input that can't physically fit in a QR code.
    let input_len = text.len();
    if input_len > MAX_INPUT_SIZE {
        let kb = input_len / 1024;
        return Err(format!(
            "input too large: {} bytes ({} KB). QR codes max out around 2,953 bytes; \
             consider splitting your data or using a smaller input.",
            input_len, kb
        ));
    }

    // Validate output size bounds.
    if size < MIN_OUTPUT_SIZE {
        return Err(format!(
            "output size too small: {} pixels. Minimum is {} (anything smaller is unscannable).",
            size, MIN_OUTPUT_SIZE
        ));
    }
    if size > MAX_OUTPUT_SIZE {
        return Err(format!(
            "output size too large: {} pixels. Maximum is {}.",
            size, MAX_OUTPUT_SIZE
        ));
    }

    // Step 1: Encode text into a QR code matrix with error correction.
    // The Version (physical size of the matrix) is auto-selected based on
    // data length and error correction level.
    let qr = QrCode::with_error_correction_level(text, ec_level)
        .map_err(|e| format!("failed to generate QR code: {e}"))?;
    let module_count = qr.width() as u32;

    match format {
        // ---------------------------------------------------------------
        // WebP: render RGBA modules, scale, encode lossless via image-webp
        // ---------------------------------------------------------------
        Format::Webp => {
            let modules = render_modules(&qr);
            let scaled = scale_nearest(&modules, module_count, size);
            // Flatten RGBA8 structs to a flat u8 array for the encoder.
            let flat: Vec<u8> = scaled.iter().flat_map(|p| [p.r, p.g, p.b, p.a]).collect();

            let mut buf = Vec::new();
            let encoder = image_webp::WebPEncoder::new(&mut buf);
            encoder
                .encode(&flat, size, size, image_webp::ColorType::Rgba8)
                .map_err(|e| format!("failed to encode WebP: {e}"))?;
            Ok(buf)
        }
        // ---------------------------------------------------------------
        // SVG: render manually from the module grid (edge-to-edge, no quiet zone)
        // ---------------------------------------------------------------
        Format::Svg => {
            let modules = render_modules(&qr);
            let scaled = scale_nearest(&modules, module_count, size);

            let mut svg = String::new();
            svg.push_str(&format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
                size, size, size, size
            ));

            // White background fills the whole canvas so SVG matches WebP/AVIF output.
            svg.push_str(&format!(
                r#"<rect x="0" y="0" width="{}" height="{}" fill="white"/>"#,
                size, size
            ));

            // Dark modules only — each becomes a 1×1 black rect at its scaled pixel position.
            // No quiet zone is added: output is edge-to-edge, consistent with WebP/AVIF.
            for dy in 0..size {
                for dx in 0..size {
                    let idx = (dy * size + dx) as usize;
                    if scaled[idx].r == 0 {
                        svg.push_str(&format!(
                            r#"<rect x="{}" y="{}" width="1" height="1" fill="black"/>"#,
                            dx, dy
                        ));
                    }
                }
            }

            svg.push_str("</svg>");
            Ok(svg.into_bytes())
        }
        // ---------------------------------------------------------------
        // AVIF: render RGBA modules, scale, encode via ravif (rav1e-based)
        // ---------------------------------------------------------------
        Format::Avif => {
            let modules = render_modules(&qr);
            let scaled = scale_nearest(&modules, module_count, size);
            let encoder = Encoder::new();
            let img = Img::new(&scaled[..], size as usize, size as usize);
            let encoded = encoder
                .encode_rgba(img)
                .map_err(|e| format!("failed to encode AVIF: {e}"))?;
            Ok(encoded.avif_file)
        }
    }
}

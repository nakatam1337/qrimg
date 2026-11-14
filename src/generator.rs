//! QR rendering + encoding into WebP / SVG / AVIF.

use crate::Format;
use qrcode::Color;
use qrcode::QrCode;
use ravif::{Encoder, Img, RGBA8};

/// Maximum input size in bytes to prevent accidentally encoding huge files.
/// QR codes top out around ~2,953 bytes (binary, Version 40, Level L), so
/// anything larger is almost certainly a mistake.
const MAX_INPUT_SIZE: usize = 2000;

/// Render the QR code into an RGBA pixel buffer (one pixel per module).
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

pub(super) fn generate_qr(text: &str, size: u32, format: Format) -> Result<Vec<u8>, String> {
    let input_len = text.len();
    if input_len > MAX_INPUT_SIZE {
        let kb = input_len / 1024;
        return Err(format!(
            "input too large: {} bytes ({} KB). QR codes max out around 2,953 bytes; \
             consider splitting your data or using a smaller input.",
            input_len, kb
        ));
    }
    let qr = QrCode::new(text).map_err(|e| format!("failed to generate QR code: {e}"))?;
    let module_count = qr.width() as u32;

    match format {
        Format::Webp => {
            let modules = render_modules(&qr);
            let scaled = scale_nearest(&modules, module_count, size);
            let flat: Vec<u8> = scaled.iter().flat_map(|p| [p.r, p.g, p.b, p.a]).collect();

            let mut buf = Vec::new();
            let encoder = image_webp::WebPEncoder::new(&mut buf);
            encoder
                .encode(&flat, size, size, image_webp::ColorType::Rgba8)
                .map_err(|e| format!("failed to encode WebP: {e}"))?;
            Ok(buf)
        }
        Format::Svg => {
            let svg = qr.render::<qrcode::render::svg::Color>().build();
            Ok(svg.into_bytes())
        }
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

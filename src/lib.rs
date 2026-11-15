//! `qrimg` core library: generate QR code images from text.
//!
//! This module contains the core logic shared by the binary and tests:
//! - [`Format`] — the output image format (WebP, SVG, AVIF)
//! - [`default_filename`] — timestamp-based default output name
//! - [`resolve_output`] — final output path resolution (format decides extension)
//! - [`generate_qr`] — public wrapper around the generator
//!
//! Supported output formats: WebP (default), SVG, AVIF.
//! The QR module matrix is rendered to RGBA pixels, then nearest-neighbor
//! scaled to the requested square output size.

use qrcode::EcLevel;
use std::time::SystemTime;

pub mod cli;
pub mod generator;

/// Output format for the generated QR code image.
///
/// Each variant carries an emoji used in the terminal spinner so users can
/// visually distinguish which format is being produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Format {
    /// WebP — lossless, widely supported, small file size. Default.
    #[default]
    Webp,
    /// SVG — vector format, scales infinitely, crisp edges.
    Svg,
    /// AVIF — modern image format, excellent compression, smaller than WebP.
    Avif,
}

impl Format {
    /// File extension for this format (without the leading dot).
    pub fn extension(self) -> &'static str {
        match self {
            Format::Webp => "webp",
            Format::Svg => "svg",
            Format::Avif => "avif",
        }
    }

    /// Emoji shown in the spinner while this format is being generated.
    pub fn emoji(self) -> &'static str {
        match self {
            Format::Webp => "🌐",
            Format::Svg => "🖼️",
            Format::Avif => "🏞️",
        }
    }
}

/// Default output file name: Unix timestamp + extension.
///
/// Using a timestamp avoids overwriting previous QR codes by accident and
/// works well when the tool is called repeatedly in a loop or script.
pub fn default_filename(format: Format) -> String {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}.{}", ts, format.extension())
}

/// Resolve the final output file name.
///
/// Resolution rules:
/// 1. If `output` is `Some`, treat it as a **base name**: strip any existing
///    extension (so the user can't accidentally force the wrong format via
///    the filename), append the extension decided by `format`, and preserve
///    any directory prefix.
/// 2. If `output` is `None`, fall back to the timestamp-based default name.
///
/// This design means `-t/--type` is the single source of truth for file
/// type, which keeps behavior predictable.
pub fn resolve_output(output: Option<&str>, format: Format) -> String {
    match output {
        Some(name) => {
            let path = std::path::Path::new(name);
            // `file_stem()` removes the final extension (e.g. "foo.svg" -> "foo")
            let base = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(name);
            let stem = format!("{}.{}", base, format.extension());
            // Preserve directory prefix if present (e.g. "/tmp/dir/foo.png")
            match path.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => {
                    parent.join(&stem).to_string_lossy().into_owned()
                }
                _ => stem,
            }
        }
        None => default_filename(format),
    }
}

/// Public API: generate QR code image bytes for `text`, scaled to `size` x `size` pixels,
/// using the given error correction level.
///
/// Delegates to [`generator::generate_qr`] which does the actual rendering
/// and encoding. Returns a `Result` so callers can handle oversized input
/// or encoding failures gracefully.
pub fn generate_qr(
    text: &str,
    size: u32,
    format: Format,
    ec_level: EcLevel,
) -> Result<Vec<u8>, String> {
    generator::generate_qr(text, size, format, ec_level)
}

#[cfg(test)]
mod tests {
    use super::{default_filename, generate_qr, resolve_output, Format};
    use crate::cli::{ec_level_to_qrcode, parse_ec_level};

    #[test]
    fn default_filename_has_correct_extension() {
        // Each format must produce a filename ending in its own extension.
        assert!(default_filename(Format::Webp).ends_with(".webp"));
        assert!(default_filename(Format::Svg).ends_with(".svg"));
        assert!(default_filename(Format::Avif).ends_with(".avif"));
    }

    #[test]
    fn default_filename_contains_timestamp() {
        // The filename should start with a Unix timestamp (10+ digits).
        let name = default_filename(Format::Webp);
        let ts_part = name.split('.').next().unwrap();
        assert!(ts_part.chars().all(|c| c.is_ascii_digit()));
        assert!(ts_part.len() >= 10);
        assert!(name.ends_with(".webp"));
    }

    #[test]
    fn format_emojis_are_non_empty() {
        // Sanity check: every format should have a display emoji.
        assert!(!Format::Webp.emoji().is_empty());
        assert!(!Format::Svg.emoji().is_empty());
        assert!(!Format::Avif.emoji().is_empty());
    }

    #[test]
    fn generate_qr_webp_returns_bytes() {
        // WebP output must start with the RIFF/WEBP container header.
        let ec = ec_level_to_qrcode("M");
        let bytes = generate_qr("hello", 64, Format::Webp, ec).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WEBP");
    }

    #[test]
    fn generate_qr_svg_returns_valid_svg() {
        // SVG output must be well-formed XML with <svg> root element.
        let ec = ec_level_to_qrcode("M");
        let bytes = generate_qr("hello", 64, Format::Svg, ec).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("<svg"));
        assert!(text.contains("</svg>"));
    }

    #[test]
    fn generate_qr_avif_returns_bytes() {
        // AVIF output must contain the AVIF "ftyp" box signature.
        let ec = ec_level_to_qrcode("M");
        let bytes = generate_qr("hello", 64, Format::Avif, ec).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[4..8], b"ftyp");
    }

    #[test]
    fn generate_qr_different_sizes_different_lengths() {
        // Larger output images should produce larger encoded files.
        let ec = ec_level_to_qrcode("M");
        let small = generate_qr("test", 32, Format::Webp, ec).unwrap().len();
        let large = generate_qr("test", 256, Format::Webp, ec).unwrap().len();
        assert!(large > small);
    }

    #[test]
    fn generate_qr_rejects_oversized_input() {
        // Input exceeding the 2,000-byte limit must be rejected with a clear error.
        let too_big = "x".repeat(2001);
        let ec = ec_level_to_qrcode("M");
        let err = generate_qr(&too_big, 64, Format::Webp, ec).unwrap_err();
        assert!(err.contains("too large"));
        assert!(err.contains("2,953"));
    }

    #[test]
    fn generate_qr_accepts_max_size_input() {
        // Input exactly at the limit should be accepted.
        let ec = ec_level_to_qrcode("M");
        let max = "x".repeat(2000);
        let bytes = generate_qr(&max, 64, Format::Webp, ec).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn resolve_output_uses_format_extension() {
        // The format flag decides the extension, not the filename argument.
        assert_eq!(resolve_output(Some("myqr"), Format::Webp), "myqr.webp");
        assert_eq!(resolve_output(Some("myqr"), Format::Svg), "myqr.svg");
        assert_eq!(resolve_output(Some("myqr"), Format::Avif), "myqr.avif");

        // Existing extensions on the filename are overridden by the format.
        assert_eq!(resolve_output(Some("foo.svg"), Format::Avif), "foo.avif");
        assert_eq!(resolve_output(Some("bar.avif"), Format::Webp), "bar.webp");

        // Directory prefixes are preserved; only the file stem changes.
        let with_dir = resolve_output(Some("/tmp/qrtest.custom.png"), Format::Webp);
        assert_eq!(with_dir, "/tmp/qrtest.custom.webp");

        // None -> timestamp default.
        let default = resolve_output(None, Format::Webp);
        assert!(default.ends_with(".webp"));
    }

    #[test]
    fn generate_qr_rejects_size_too_small() {
        // Size below 8 pixels must be rejected.
        let ec = ec_level_to_qrcode("M");
        let err = generate_qr("test", 7, Format::Webp, ec).unwrap_err();
        assert!(err.contains("too small"));
    }

    #[test]
    fn generate_qr_rejects_size_too_large() {
        // Size above 8192 pixels must be rejected.
        let ec = ec_level_to_qrcode("M");
        let err = generate_qr("test", 8193, Format::Webp, ec).unwrap_err();
        assert!(err.contains("too large"));
    }

    #[test]
    fn generate_qr_all_ec_levels() {
        // All four error correction levels must produce valid output.
        for level in ["L", "M", "Q", "H"] {
            let ec = ec_level_to_qrcode(level);
            let bytes = generate_qr("hello", 64, Format::Webp, ec).unwrap();
            assert!(!bytes.is_empty(), "EC level {level} produced empty output");
        }
    }

    #[test]
    fn generate_qr_svg_has_white_background() {
        // SVG output must include a full-canvas white background rect.
        let ec = ec_level_to_qrcode("M");
        let bytes = generate_qr("hello", 64, Format::Svg, ec).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("<rect x=\"0\" y=\"0\"") && text.contains("fill=\"white\"/>"),
            "SVG should have a white background rect"
        );
    }

    #[test]
    fn parse_ec_level_accepts_all_levels() {
        // All four levels (case-insensitive) must parse successfully.
        for input in ["L", "M", "Q", "H", "l", "m", "q", "h"] {
            let result = parse_ec_level(input);
            assert!(result.is_ok(), "parse_ec_level('{input}') should succeed");
            assert_eq!(result.unwrap(), input.to_uppercase());
        }
    }

    #[test]
    fn parse_ec_level_rejects_invalid() {
        // Invalid EC level strings must be rejected.
        for input in ["X", "LM", "", "invalid", "l m"] {
            let result = parse_ec_level(input);
            assert!(result.is_err(), "parse_ec_level('{input}') should fail");
        }
    }

    #[test]
    fn ec_level_to_qrcode_maps_all_levels() {
        // Each string level must map to the correct qrcode::EcLevel variant.
        assert_eq!(ec_level_to_qrcode("L"), qrcode::EcLevel::L);
        assert_eq!(ec_level_to_qrcode("M"), qrcode::EcLevel::M);
        assert_eq!(ec_level_to_qrcode("Q"), qrcode::EcLevel::Q);
        assert_eq!(ec_level_to_qrcode("H"), qrcode::EcLevel::H);
    }
}

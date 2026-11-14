//! `qrimg` core library: generate QR code images from text.
//!
//! Supported output formats: WebP (default), SVG, AVIF.
//! The QR module matrix is rendered then nearest-neighbor scaled to the
//! requested square output size.

pub mod cli;
pub mod generator;

use std::time::SystemTime;

/// Output format for the generated QR code image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Format {
    #[default]
    Webp,
    Svg,
    Avif,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Webp => "webp",
            Format::Svg => "svg",
            Format::Avif => "avif",
        }
    }

    pub fn emoji(self) -> &'static str {
        match self {
            Format::Webp => "🌐",
            Format::Svg => "🖼️",
            Format::Avif => "🏞️",
        }
    }
}

/// Default output file name: Unix timestamp + extension.
pub fn default_filename(format: Format) -> String {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}.{}", ts, format.extension())
}

/// Public API: generate QR code image bytes for `text`, scaled to `size` x `size` pixels.
pub fn generate_qr(text: &str, size: u32, format: Format) -> Result<Vec<u8>, String> {
    generator::generate_qr(text, size, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filename_has_correct_extension() {
        assert!(default_filename(Format::Webp).ends_with(".webp"));
        assert!(default_filename(Format::Svg).ends_with(".svg"));
        assert!(default_filename(Format::Avif).ends_with(".avif"));
    }

    #[test]
    fn default_filename_contains_timestamp() {
        let name = default_filename(Format::Webp);
        let ts_part = name.split('.').next().unwrap();
        assert!(ts_part.chars().all(|c| c.is_ascii_digit()));
        assert!(ts_part.len() >= 10);
        assert!(name.ends_with(".webp"));
    }

    #[test]
    fn format_emojis_are_non_empty() {
        assert!(!Format::Webp.emoji().is_empty());
        assert!(!Format::Svg.emoji().is_empty());
        assert!(!Format::Avif.emoji().is_empty());
    }

    #[test]
    fn generate_qr_webp_returns_bytes() {
        let bytes = generate_qr("hello", 64, Format::Webp).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WEBP");
    }

    #[test]
    fn generate_qr_svg_returns_valid_svg() {
        let bytes = generate_qr("hello", 64, Format::Svg).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("<svg"));
        assert!(text.contains("</svg>"));
    }

    #[test]
    fn generate_qr_avif_returns_bytes() {
        let bytes = generate_qr("hello", 64, Format::Avif).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[4..8], b"ftyp");
    }

    #[test]
    fn generate_qr_different_sizes_different_lengths() {
        let small = generate_qr("test", 32, Format::Webp).unwrap().len();
        let large = generate_qr("test", 256, Format::Webp).unwrap().len();
        assert!(large > small);
    }
}

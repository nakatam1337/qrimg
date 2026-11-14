//! `qrimg` core library: generate QR code images from text.
//!
//! Supported output formats: WebP (default), SVG, AVIF.
//! The QR module matrix is rendered then nearest-neighbor scaled to the
//! requested square output size.

pub mod cli;
pub mod generator;

use std::path::Path;
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

/// Resolve the final output file name.
///
/// If `output` is `Some`, use its file stem (stripping any existing
/// extension) and append the extension decided by `format`, keeping any
/// directory prefix intact.
/// If `output` is `None`, fall back to the timestamp-based default name.
pub fn resolve_output(output: Option<&str>, format: Format) -> String {
    match output {
        Some(name) => {
            let path = Path::new(name);
            let base = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(name);
            let stem = format!("{}.{}", base, format.extension());
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
    fn resolve_output_uses_format_extension() {
        // -o name without extension -> format extension appended
        assert_eq!(resolve_output(Some("myqr"), Format::Webp), "myqr.webp");
        assert_eq!(resolve_output(Some("myqr"), Format::Svg), "myqr.svg");
        assert_eq!(resolve_output(Some("myqr"), Format::Avif), "myqr.avif");

        // -o name with extension -> extension overridden by format
        assert_eq!(resolve_output(Some("foo.svg"), Format::Avif), "foo.avif");
        assert_eq!(resolve_output(Some("bar.avif"), Format::Webp), "bar.webp");

        // None -> timestamp default
        let default = resolve_output(None, Format::Webp);
        assert!(default.ends_with(".webp"));

        // Directory prefix is preserved, final extension replaced by format
        let with_dir = resolve_output(Some("/tmp/qrtest.custom.png"), Format::Webp);
        assert_eq!(with_dir, "/tmp/qrtest.custom.webp");
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

    #[test]
    fn generate_qr_rejects_oversized_input() {
        let too_big = "x".repeat(2001);
        let err = generate_qr(&too_big, 64, Format::Webp).unwrap_err();
        assert!(err.contains("too large"));
        assert!(err.contains("2,953"));
    }

    #[test]
    fn generate_qr_accepts_max_size_input() {
        let max = "x".repeat(2000);
        let bytes = generate_qr(&max, 64, Format::Webp).unwrap();
        assert!(!bytes.is_empty());
    }
}

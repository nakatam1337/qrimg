//! Command-line argument parsing for `qrimg`.
//!
//! This module defines the [`Args`] struct using clap's derive API.
//! All flags, their short/long names, defaults, and help text live here.

use clap::Parser;
use crate::Format;

/// Generate QR code images (.webp, .svg, .avif) from text on the command line.
///
/// Input priority chain (highest to lowest):
/// 1. `-m / --message` — explicit text argument
/// 2. `-f / --file` — read text from a file
/// 3. stdin — interactive prompt (Ctrl+D to create, Ctrl+C to cancel)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Message to encode into a QR code.
    ///
    /// If omitted, the tool falls back to `-f` (file) or stdin.
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,

    /// Read message from a text file (takes priority over `-m` and stdin).
    ///
    /// Useful for encoding the contents of a file without shell quoting issues.
    #[arg(short = 'f', long = "file")]
    pub file: Option<String>,

    /// Output file name.
    ///
    /// Only the base name matters — the extension is decided by `-t/--type`.
    /// Default: Unix timestamp + extension (e.g. `1794695770.webp`).
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// Output format: webp (default), svg, or avif.
    ///
    /// This flag is the single source of truth for file type.
    #[arg(
        short = 't',
        long = "type",
        default_value = "webp",
        value_enum
    )]
    pub format: Format,

    /// Output image size in pixels (square).
    ///
    /// QR codes are square, so only one side needs to be specified.
    /// Valid range: 8–8192. Default: 512.
    #[arg(short = 's', long = "size", default_value = "512")]
    pub size: u32,

    /// Reed-Solomon error correction level.
    ///
    /// Higher levels add redundancy so the QR code remains scannable even if
    /// partially damaged, but reduce the amount of data it can store.
    /// Levels: L (~7%), M (~15%), Q (~25%), H (~30%) redundancy.
    /// Default: M.
    #[arg(short = 'e', long = "ec-level", default_value = "M", value_parser = parse_ec_level)]
    pub ec_level: String,

    /// Skip overwrite confirmation and force-write the output file.
    ///
    /// Use this in scripts or when you are certain you want to overwrite
    /// an existing file without being prompted.
    #[arg(short = 'y', long = "force")]
    pub force: bool,
}

/// Parse a user-supplied error correction level string into a `qrcode::EcLevel`.
///
/// Accepted values: `L`, `M`, `Q`, `H` (case-insensitive).
pub fn parse_ec_level(s: &str) -> Result<String, String> {
    let upper = s.to_uppercase();
    match upper.as_str() {
        "L" | "M" | "Q" | "H" => Ok(upper),
        _ => Err(format!(
            "invalid ec-level '{s}'. Use one of: L, M, Q, H"
        )),
    }
}

/// Convert a validated ec-level string into a `qrcode::EcLevel`.
pub fn ec_level_to_qrcode(level: &str) -> qrcode::EcLevel {
    match level {
        "L" => qrcode::EcLevel::L,
        "M" => qrcode::EcLevel::M,
        "Q" => qrcode::EcLevel::Q,
        "H" => qrcode::EcLevel::H,
        _ => qrcode::EcLevel::M, // unreachable if parse_ec_level was used
    }
}

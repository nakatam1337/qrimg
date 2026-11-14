//! Command-line argument parsing for `qrimg`.

use clap::Parser;
use crate::Format;

/// Generate QR code images (.webp, .svg, .avif) from text on the command line.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Message to encode into a QR code. If omitted, reads from a file (-f) or stdin.
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,

    /// Read message from a text file (takes priority over -m and stdin).
    #[arg(short = 'f', long = "file")]
    pub file: Option<String>,

    /// Output file name. Default: unix timestamp + extension (e.g. 1794695770.webp)
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// Output format: webp (default), svg, or avif.
    #[arg(short = 't', long = "type", default_value = "webp", value_enum)]
    pub format: Format,

    /// Output image size in pixels (square). Default: 512.
    #[arg(short = 's', long = "size", default_value = "512")]
    pub size: u32,
}

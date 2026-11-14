//! `qrimg` binary entry point.

use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use clap::Parser;
use qrimg::cli::Args;
use qrimg::{default_filename, generate_qr};

fn main() {
    let args = Args::parse();
    let format = args.format;
    let size = args.size;

    let text = match args.message {
        Some(m) => m,
        None => read_stdin_with_prompt(),
    };

    let out_name = args.output.unwrap_or_else(|| default_filename(format));

    // Animated spinner while generating.
    let spinning = Arc::new(AtomicBool::new(true));
    let spinning_clone = Arc::clone(&spinning);
    let format_emoji = format.emoji();
    let spinner = thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        while spinning_clone.load(Ordering::Relaxed) {
            let _ = write!(io::stderr(), "\r{} generating QR... {}", frames[i % frames.len()], format_emoji);
            let _ = io::stderr().flush();
            i = (i + 1) % frames.len();
            thread::sleep(std::time::Duration::from_millis(80));
        }
        let _ = write!(io::stderr(), "\r{} done.   \n", format_emoji);
        let _ = io::stderr().flush();
    });

    let bytes = match generate_qr(&text, size, format) {
        Ok(b) => b,
        Err(e) => {
            spinning.store(false, Ordering::Relaxed);
            let _ = spinner.join();
            eprintln!("❌ {e}");
            std::process::exit(1);
        }
    };

    spinning.store(false, Ordering::Relaxed);
    let _ = spinner.join();

    let path = std::path::Path::new(&out_name);
    let mut file = std::fs::File::create(path).unwrap_or_else(|_| {
        eprintln!("❌ failed to create {}", out_name);
        std::process::exit(1);
    });
    file.write_all(&bytes).unwrap_or_else(|_| {
        eprintln!("❌ failed to write {}", out_name);
        std::process::exit(1);
    });

    println!("✅ {}  ({} bytes, {}×{})", out_name, bytes.len(), size, size);
}

/// Read text from stdin with a newline-prefixed emoji prompt.
/// Ctrl+D creates the QR; Ctrl+C cancels.
fn read_stdin_with_prompt() -> String {
    let mut buf = String::new();
    let _ = write!(io::stderr(), "\n🔲 Enter text (Ctrl+D to create, Ctrl+C to cancel): ");
    let _ = io::stderr().flush();
    io::stdin()
        .read_to_string(&mut buf)
        .expect("failed to read stdin");
    buf.trim_end().to_string()
}

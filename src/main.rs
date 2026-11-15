//! `qrimg` binary entry point.
//!
//! This module ties everything together:
//! - Parses CLI arguments ([`crate::cli::Args`])
//! - Reads input text (message > file > stdin)
//! - Shows an animated spinner while generating
//! - Calls [`crate::generate_qr`] to produce image bytes
//! - Handles overwrite confirmation before writing the output file

use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use clap::Parser;
use qrimg::cli::{ec_level_to_qrcode, Args};
use qrimg::{generate_qr, resolve_output};

fn main() {
    // Parse CLI arguments using clap's derive API.
    let args = Args::parse();
    let format = args.format;
    let size = args.size;

    // Step 1: Resolve input text following the priority chain:
    //   -m message > -f file > stdin prompt
    let text = read_input(&args);

    // Step 2: Resolve output filename (format decides the extension).
    let out_name = resolve_output(args.output.as_deref(), format);

    // Step 3: Overwrite confirmation.
    // If the output file already exists and --force was not passed, ask the
    // user for confirmation unless we are in a non-interactive (piped)
    // context, in which case we fail with a clear message.
    let path = Path::new(&out_name);
    if path.exists() && !args.force {
        if std::io::stdout().is_terminal() {
            // Interactive terminal — prompt for confirmation.
            print!("⚠️  '{}' already exists. Overwrite? (y/N) ", out_name);
            io::stdout().flush().unwrap();
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            let answer = answer.trim().to_lowercase();
            if answer != "y" && answer != "yes" {
                println!("❌ Aborted: output file not overwritten.");
                std::process::exit(0);
            }
        } else {
            // Non-interactive (piped/redirected) — refuse to overwrite silently.
            eprintln!(
                "❌ output file '{}' already exists. Use -y/--force to overwrite.",
                out_name
            );
            std::process::exit(1);
        }
    }

    // -------------------------------------------------------------------
    // Animated spinner while generating.
    // -------------------------------------------------------------------
    // We run the spinner on a separate thread so the terminal shows activity
    // while QR encoding happens on the main thread. An Arc<AtomicBool> is
    // used as a cross-thread cancellation flag.
    let spinning = Arc::new(AtomicBool::new(true));
    let spinning_clone = Arc::clone(&spinning);
    let format_emoji = format.emoji();
    let spinner = thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        while spinning_clone.load(Ordering::Relaxed) {
            // `\r` returns to line start, `\x1b[2K` clears the whole line.
            // This prevents leftover characters from longer frames.
            let _ = write!(
                io::stderr(),
                "\r\x1b[2K{} generating QR... {}",
                frames[i % frames.len()],
                format_emoji
            );
            let _ = io::stderr().flush();
            i = (i + 1) % frames.len();
            thread::sleep(std::time::Duration::from_millis(80));
        }
        // Final "done" message, then newline so subsequent output starts fresh.
        let _ = write!(io::stderr(), "\r\x1b[2K{} done.\n", format_emoji);
        let _ = io::stderr().flush();
    });

    // Step 4: Generate the QR code image bytes.
    let ec = ec_level_to_qrcode(&args.ec_level);
    let bytes = match generate_qr(&text, size, format, ec) {
        Ok(b) => b,
        Err(e) => {
            // Stop the spinner before printing the error so it doesn't linger.
            spinning.store(false, Ordering::Relaxed);
            let _ = spinner.join();
            eprintln!("❌ {e}");
            std::process::exit(1);
        }
    };

    // Step 5: Stop the spinner and wait for the thread to finish.
    spinning.store(false, Ordering::Relaxed);
    let _ = spinner.join();

    // Step 6: Write the encoded bytes to the output file.
    let mut file = std::fs::File::create(path).unwrap_or_else(|_| {
        eprintln!("❌ failed to create {}", out_name);
        std::process::exit(1);
    });
    file.write_all(&bytes).unwrap_or_else(|_| {
        eprintln!("❌ failed to write {}", out_name);
        std::process::exit(1);
    });

    // Step 7: Report success to stdout.
    println!("✅ {}  ({} bytes, {}×{})", out_name, bytes.len(), size, size);
}

/// Resolve the input text following the priority chain:
/// `-f` file takes priority, then `-m` message, then stdin.
fn read_input(args: &Args) -> String {
    if let Some(path) = &args.file {
        // Read from file — useful for piping file contents without shell quoting.
        std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("❌ failed to read file '{path}': {e}");
            std::process::exit(1);
        })
    } else if let Some(m) = &args.message {
        // Explicit message argument.
        m.clone()
    } else {
        // Interactive: prompt the user and read from stdin.
        read_stdin_with_prompt()
    }
}

/// Read text from stdin with a newline-prefixed emoji prompt.
///
/// Behavior:
/// - Prints a prompt to stderr so it doesn't mix with piped stdout data
/// - User types/pastes text, then presses Ctrl+D (EOF) to create the QR
/// - Ctrl+C cancels (handled by the OS as SIGINT)
/// - Trailing newline is trimmed before returning
fn read_stdin_with_prompt() -> String {
    let mut buf = String::new();
    // Prompt goes to stderr so it doesn't mix with piped stdout data.
    // Trailing newline puts the cursor on the next line for input.
    let _ = write!(io::stderr(), "🔲 Enter text (Ctrl+D to create, Ctrl+C to cancel):\n");
    let _ = io::stderr().flush();
    io::stdin()
        .read_to_string(&mut buf)
        .expect("failed to read stdin");
    buf.trim_end().to_string()
}



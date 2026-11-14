# AGENTS.md

## Project
`qrimg` — generate QR code images (`.webp`, `.svg`, `.avif`) from text on the command line.

## Structure
- `src/lib.rs` — crate root, `Format` enum, `default_filename()`, `resolve_output()`, `generate_qr()` public wrapper, unit tests
- `src/cli.rs` — clap argument parsing (`Args`)
- `src/generator.rs` — QR module rendering, nearest-neighbor scaling, WebP/SVG/AVIF encoding, input size limit
- `src/main.rs` — binary entry point, stdin prompt with emoji, animated spinner, file writing

## Conventions
- **`-f, --file`** — read message from a text file (priority over `-m` and stdin)
- **`-m, --message`** — message argument
- **`-t, --type`** — output format: `webp` (default), `svg`, `avif`
- **`-o, --output`** — output filename (base name only; extension is decided by `-t`)
- **`-s, --size`** — square image size in pixels, default `512`
- **Input limit** — 2,000 bytes max. QR codes top out around 2,953 bytes; larger input is rejected with a clear error.
- **Default output name** — Unix timestamp + extension (e.g. `1794695770.webp`)

## Commands
```bash
cargo build --release      # build release binary
cargo test                 # run unit tests (lib + bin)
cargo install --path . --force  # install locally
cargo publish              # publish to crates.io (needs CARGO_REGISTRY_TOKEN)
```

## Publishing
Set `CARGO_REGISTRY_TOKEN` environment variable with your crates.io API token, then run `cargo publish`. Do not commit the token.

## Notes
- WebP output uses `image-webp` (pure Rust, lossless)
- AVIF output uses `ravif` (pure Rust, rav1e-based, needs `nasm` at build time for `asm` feature)
- SVG output uses `qrcode`'s built-in `svg` feature
- Spinner uses ANSI escape `\r\x1b[2K` to clear the line cleanly
- Input priority: `-m` > `-f` > stdin (with `Ctrl+D` to create, `Ctrl+C` to cancel)

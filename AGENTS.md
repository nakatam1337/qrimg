# AGENTS.md

## Project
`qrimg` — generate QR code images (`.webp`, `.svg`, `.avif`) from text on the command line.

## Structure
- `src/lib.rs` — crate root, `Format` enum, `default_filename()`, `resolve_output()`, `generate_qr()` public wrapper, unit tests
- `src/cli.rs` — clap argument parsing (`Args`), `parse_ec_level()`, `ec_level_to_qrcode()`
- `src/generator.rs` — QR module rendering, nearest-neighbor scaling, WebP/SVG/AVIF encoding, input + size validation
- `src/main.rs` — binary entry point, stdin prompt with emoji, animated spinner, overwrite confirmation, file writing
- `tests/cli.rs` — CLI integration tests (spawn the compiled binary)
- `LICENSE` — CC0-1.0 (public domain dedication)
- `README.md` — user-facing documentation
- `AGENTS.md` — this file
- `ROADMAP.md` — planned features

## Conventions
- **`-m, --message`** — text to encode (priority over `-f` and stdin)
- **`-f, --file`** — read message from a text file
- **`-t, --type`** — output format: `webp` (default), `svg`, `avif`
- **`-o, --output`** — output filename (base name only; extension decided by `-t`)
- **`-s, --size`** — square image size in pixels, default `512`, range `8–8192`
- **`-e, --ec-level`** — error correction: `L`, `M` (default), `Q`, `H`
- **`-y, --force`** — skip overwrite confirmation
- **Input limit** — 2,000 bytes max. QR codes top out around 2,953 bytes; larger input is rejected.
- **Overwrite** — if output exists and `-y` not passed, prompts (interactive) or fails (non-interactive)
- **Default output name** — Unix timestamp + extension (e.g. `1794695770.webp`)

## Commands
```bash
cargo build --release      # build release binary
cargo test                 # run unit tests (lib + bin) + CLI integration tests
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
- `EcLevel` is passed as `String` from CLI, parsed via `parse_ec_level()`, converted via `ec_level_to_qrcode()`

## Roadmap (see ROADMAP.md)
- Quiet zone (padding around modules)
- Custom colors (foreground/background)
- Center logo / image embedding
- Batch mode (multiple QR codes from file)
